start-testnode:
	anvil

watch-tailwind:
	rm templates/static/output.css
	npx tailwindcss -i ./templates/input.css -o ./templates/static/output.css --watch
