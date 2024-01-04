> Broken atm!

Simple example that runs [Mistral](https://mistral.ai/news/announcing-mistral-7b/) model using [candle](https://github.com/huggingface/candle) framework. Adopted from [candle's mistral example](https://github.com/huggingface/candle/tree/main/candle-examples/examples/mistral), but without platform-dependent optimizations and with UI for the easiest start.

Candle is a framework developed by [Hugging Face](https://huggingface.co/) - leading AI platform in the world. They started to build it because Python, common choice for AI development, introduces significant performance and devops overhead, while rust solves these problems, enchances reliability and provides direct access to WASM and WebGPU ecosystems to easily run models on the client side. As of now it's not as easy to use as PyTorch and missing some important features, but the future is bright and it already supports a lot of modern models like the one used in this example.

As always let's start with the manifest:

{Cargo.toml}

It includes `hf-hub` that simplifies model loading, `tokenizers` - another hugging face utility for efficient text pre- and postprocessing for LLMs, and `candle-*` crates which run calculations of the models. 

The core example's code is in:

{llm.rs}

It defines how the model is initialized, encodes, performs inference and decodes. Prest-based service that works with this model is defined here:

{serve.rs}

Beware that it's a simple and naive implementation designed to check it out locally. For real-world SaaS or other types of services model should be managed differently, but this example is enough to demonstrate core building blocks.