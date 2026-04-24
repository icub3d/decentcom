# decentcom-chatbot

A reference chatbot for decentcom that connects to an LLM provider (like Ollama) to answer questions and chat with users. It listens for mentions in channels it has access to and responds with AI-generated text.

## Features
- Responds to `@chatbot` (or its configured display name).
- Configurable LLM provider (currently supports Ollama).
- Strips its own mention from prompts before processing.

## Setup

1. Copy the example configuration:
   ```bash
   cp chatbot.toml.example chatbot.toml
   ```

2. Edit `chatbot.toml` with your server URL, mnemonic (from registering the bot), and Ollama settings.

3. Make sure Ollama is running locally (or remotely) and has the configured model available:
   ```bash
   ollama run llama3
   ```

4. Run the bot:
   ```bash
   BOT_CONFIG=chatbot.toml cargo run --bin chatbot
   ```

## Configuration (Ollama)

By default, the bot connects to `http://127.0.0.1:11434` to communicate with Ollama. Make sure Ollama is running and accessible on this URL. You can change the model, system prompt, and temperature in the TOML configuration file.
