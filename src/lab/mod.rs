pub mod smart_money;
pub mod the_blockchain_messenger;

pub use smart_money::main::setup_handlers as smart_money_handlers;
pub use the_blockchain_messenger::main::setup_handlers as the_blockchain_messenger_handlers;
