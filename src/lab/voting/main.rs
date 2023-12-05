use std::{
    fmt::Debug,
    future::{self, Ready},
    sync::Arc,
};

use crate::{
    app::model::State as AppState,
    helper,
    lab::{deploy, load_template},
};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use ethers::{
    abi::{Address, FixedBytes, Token},
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Provider, Ws},
    signers::{LocalWallet, Wallet},
    types::{H160, U256},
};
use ethers_contract::{Contract, EthEvent, LogMeta};
use futures::{join, StreamExt};
use k256::Secp256k1;
use serde::Deserialize;
use tera::Context;

#[derive(Deserialize, Debug)]
enum Action {
    GiveRightToVote,
    Delegate,
    Vote,
    Delete,
}

#[derive(Deserialize, Debug)]
struct FormData {
    action: Action,
    address: Option<String>,
    proposal: Option<u8>,
}

#[derive(Debug, Clone, EthEvent)]
struct BallotCreated {
    #[ethevent(indexed, name = "_chairperson")]
    chairperson: Address,
    #[ethevent(name = "_proposals")]
    proposals: String,
}
#[derive(Debug, Clone, EthEvent)]
struct GotRightToVote {
    #[ethevent(indexed, name = "_voter")]
    voter: Address,
}

#[derive(Debug, Clone, EthEvent)]
struct RightDelegated {
    #[ethevent(indexed, name = "_from")]
    from: Address,
    #[ethevent(indexed, name = "_to")]
    to: Address,
}

#[derive(Debug, Clone, EthEvent)]
struct Voted {
    #[ethevent(indexed, name = "_voter")]
    voter: Address,
    #[ethevent(indexed, name = "_proposal")]
    proposal: U256,
    #[ethevent(name = "_weight")]
    weight: U256,
}

abigen!(
    Ballot,
    r#"[
        event BallotCreated(address indexed _chairperson, string _proposals)
        event GotRightToVote(address indexed _voter)
        event RightDelegated(address indexed _from, address indexed _to)
        event Voted(address indexed _voter, uint indexed _proposal, uint _weight)

        function giveRightToVote(address)
        function delegate(address)
        function vote(uint)
        function winningProposal()(uint)
        function winnerName()(bytes32) 
        function deleteBallot()()

        function chairperson()(address)
        function getProposalsAsString()(string)
    ]"#
);

const CONTRACT_NAME: &str = "Ballot";
const LAB_PATH: &str = "lab/voting";
const LAB_BASEURL: &str = "/lab/voting";
const CONTRACT_ADDRESS_ENVVAR: &str = "CONTRACT_ADDRESS_VOTING";
const BALLOT_PROPOSAL_NAMES_ENVVAR: &str = "BALLOT_PROPOSAL_NAMES";

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource(LAB_BASEURL).route(web::get().to(load_template_handler)))
        .service(
            web::resource(format!("{}/result", LAB_BASEURL))
                .route(web::get().to(tx_result_handler)),
        )
        .service(
            web::resource(format!("{}/deploy", LAB_BASEURL)).route(web::post().to(deploy_handler)),
        )
        .service(
            web::resource(format!("{}/form", LAB_BASEURL))
                .route(web::get().to(override_lab_handler)),
        )
        .service(
            web::resource(format!("{}/submit", LAB_BASEURL)).route(web::post().to(submit_handler)),
        );
}

async fn load_template_handler(app_state: web::Data<AppState>) -> impl Responder {
    load_template(app_state, LAB_PATH, CONTRACT_NAME).await
}

async fn override_lab_handler(app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("other_account_addresses", &app_state.accounts[1..]);

    let lock = app_state.contracts.read().await;
    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };
    let contract = Ballot::new(contract.address(), contract.client());

    let proposal_votes = match contract.get_proposals_as_string().call().await {
        Ok(proposals) => proposals,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    let proposals: Vec<&str> = proposal_votes
        .split('\n')
        .map(|p| match p.rfind(" => ") {
            Some(pos) => p[..pos].trim(),
            None => p.trim(),
        })
        .filter(|&p| !p.is_empty())
        .collect();
    context.insert("proposals", &proposals);

    let file_name = "lab/voting/form.html".to_owned();
    match app_state.tmpl.render(&file_name, &context) {
        Ok(rendered) => HttpResponse::Ok()
            .append_header(("HX-Trigger", "loadResult"))
            .body(rendered),
        Err(e) => helper::render_error(e),
    }
}

async fn tx_result_handler(app_state: web::Data<AppState>) -> impl Responder {
    let result_path = format!("{}/result.html", LAB_PATH);

    let lock = app_state.contracts.read().await;

    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };
    let contract_address = format!("{:#x}", contract.address());

    let mut context = Context::new();
    context.insert("contract_address", &contract_address);

    let contract = Ballot::new(contract.address(), contract.client());

    let chairperson = match contract.chairperson().call().await {
        Ok(chairperson) => format!("{:#x}", chairperson),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("chairperson", &chairperson);

    let winner_name = match contract.winner_name().call().await {
        Ok(name) => name,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    let winner_name = match ethers::utils::parse_bytes32_string(&winner_name) {
        Ok(name) => name,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("winner_name", &winner_name);

    let proposal_votes = match contract.get_proposals_as_string().call().await {
        Ok(proposals) => proposals.replace('\n', "<br/>"),
        Err(e) => return helper::ui_alert(&e.to_string()),
    };
    context.insert("proposal_votes", &proposal_votes);

    let rendered = match app_state.tmpl.render(&result_path, &context) {
        Ok(rendered) => rendered,
        Err(e) => return helper::render_error(e),
    };

    HttpResponse::Ok().body(rendered)
}

async fn deploy_handler(app_state: web::Data<AppState>) -> HttpResponse {
    let proposals = match helper::get_env_var(BALLOT_PROPOSAL_NAMES_ENVVAR) {
        Ok(proposals) => proposals,
        Err(e) => return helper::ui_alert(&e.to_string()),
    };

    let proposals = proposals
        .split('\n')
        .map(|p| p.trim())
        .filter(|&p| !p.is_empty())
        .map(|p| Token::FixedBytes(FixedBytes::from(p)))
        .collect();

    deploy(
        app_state.clone(),
        CONTRACT_NAME,
        CONTRACT_ADDRESS_ENVVAR,
        LAB_BASEURL,
        Token::Array(proposals),
    )
    .await
}

type EventResult<T> = Result<
    (T, LogMeta),
    ethers_contract::ContractError<
        SignerMiddleware<ethers_providers::Provider<Ws>, Wallet<ecdsa::SigningKey<Secp256k1>>>,
    >,
>;
fn log_events<T: EthEvent + Debug + Clone>(event: EventResult<T>) -> Ready<()> {
    match event {
        Ok(event) => log::debug!("••• Received event: {event:?}"),
        Err(e) => log::error!("••• Error receiving event: {e:?}"),
    };
    future::ready(())
}

pub async fn subscribe_to_events(eth_client: Arc<SignerMiddleware<Provider<Ws>, LocalWallet>>) {
    let ballot_created_future = async {
        let ballot_created = Contract::event_of_type::<BallotCreated>(eth_client.clone());
        let ballot_created_subscription = ballot_created.subscribe_with_meta().await.unwrap();
        ballot_created_subscription.for_each(log_events).await;
    };

    let got_right_to_vote_future = async {
        let got_right_to_vote = Contract::event_of_type::<GotRightToVote>(eth_client.clone());
        let got_right_to_vote_subscription = got_right_to_vote.subscribe_with_meta().await.unwrap();
        got_right_to_vote_subscription.for_each(log_events).await;
    };

    let right_delegated_future = async {
        let right_delegated = Contract::event_of_type::<RightDelegated>(eth_client.clone());
        let right_delegated_subscription = right_delegated.subscribe_with_meta().await.unwrap();
        right_delegated_subscription.for_each(log_events).await;
    };

    let voted_future = async {
        let voted = Contract::event_of_type::<Voted>(eth_client.clone());
        let voted_subscription = voted.subscribe_with_meta().await.unwrap();
        voted_subscription.for_each(log_events).await;
    };

    join!(
        ballot_created_future,
        got_right_to_vote_future,
        right_delegated_future,
        voted_future
    );
}

async fn submit_handler(
    form: web::Form<FormData>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    app_state
        .debug_service
        .send_debug_event(&format!(
            "<b>[{CONTRACT_NAME}]</b> transaction requested: {form:?}"
        ))
        .await;

    let lock = app_state.contracts.read().await;
    let contract = match lock.get(CONTRACT_NAME) {
        Some(contract) => contract,
        None => return helper::ui_alert(&format!("contract {} not deployed", CONTRACT_NAME)),
    };

    let contract = Ballot::new(contract.address(), contract.client());
    let adr = if form.address.is_none() {
        H160::zero()
    } else {
        match helper::parse_address(form.address.clone().unwrap().as_str()) {
            Ok(adr) => adr,
            Err(e) => return helper::ui_alert(&e.to_string()),
        }
    };
    let proposal = U256::from(form.proposal.unwrap_or(0));

    let call = match form.action {
        Action::GiveRightToVote => contract.give_right_to_vote(adr),
        Action::Delegate => contract.delegate(adr),
        Action::Vote => contract.vote(proposal - 1),
        Action::Delete => contract.delete_ballot(),
    };

    let pending_tx = match call.send().await {
        Ok(receipt) => receipt,
        Err(e) => {
            let err = match e.as_revert() {
                Some(err) => helper::decode_revert_error(err),
                None => e.to_string(),
            };
            return helper::ui_alert(&err);
        }
    };

    match pending_tx.await {
        Ok(receipt) => {
            app_state
                .debug_service
                .send_debug_event(&format!("<b>[{CONTRACT_NAME}]</b> receipt: {receipt:?}"))
                .await;
            helper::trigger_reload()
        }
        Err(e) => helper::ui_alert(&e.to_string()),
    }
}
