use anyhow::{Error as E, Result};
use candle_transformers::{
    generation::LogitsProcessor, 
    models::quantized_mistral::{Config, Model},
    utils::apply_repeat_penalty,
    quantized_var_builder::VarBuilder,
};
use tokenizers::Tokenizer;

static SEED: u64 = 123456789;
/// Penalty to be applied for repeating tokens, 1. means no penalty.
static REPEAT_PENALTY: f32 = 1.1;
/// The context size to consider for the repeat penalty.
static REPEAT_LAST_N: usize = 64;

pub struct Mistral {
    model: Model,
    tokenizer: Tokenizer,
    eos_token: u32,
    logits_processor: LogitsProcessor,
}

impl Mistral {
    pub fn new() -> Result<Self> {
        let start = std::time::Instant::now();
        let repo = hf_hub::Repo::model("lmz/candle-mistral".to_owned());
        let repo_api = hf_hub::api::sync::Api::new()?.repo(repo);
        let tokenizer_filename = repo_api.get("tokenizer.json")?;
        let weights_filename = repo_api.get("model-q4k.gguf")?;
        println!("retrieved the files in {:?}", start.elapsed());
    
        let start = std::time::Instant::now();
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;
        let model = Model::new(&Config::config_7b_v0_1(true), VarBuilder::from_gguf(&weights_filename)?)?;
        let logits_processor = LogitsProcessor::new(SEED, None, None);
        println!("loaded the model in {:?}", start.elapsed());

        let eos_token = tokenizer.get_vocab(true).get("</s>").copied().unwrap();
        
        Ok(Self {
            model,
            tokenizer,
            eos_token,
            logits_processor,
        })
    }

    pub fn sample(&mut self, prompt: &str, sample_len: usize) -> Result<()> {
        let start = std::time::Instant::now();
        let mut tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();

        let mut prev_index = tokens.len();
        let mut current_index = tokens.len();
        
        for index in 0..sample_len {
            let start_pos = if index == 0 || tokens.len() == 0 { 0 } else { tokens.len() - 1 };
            let penalty_pos = tokens.len().saturating_sub(REPEAT_LAST_N);
            let input = candle::Tensor::new(&tokens[start_pos..], &candle::Device::Cpu)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(candle::DType::F32)?;
            let logits = apply_repeat_penalty(
                    &logits,
                    REPEAT_PENALTY,
                    &tokens[penalty_pos..],
            )?;
            let next_token = self.logits_processor.sample(&logits)?;
            
            if next_token == self.eos_token {
                break;
            }

            tokens.push(next_token);
            let processed_text = self.tokenizer.decode(&tokens[prev_index..current_index], true).unwrap();
            let updated_text = self.tokenizer.decode(&tokens[prev_index..], true).unwrap();
            if updated_text.len() > processed_text.len() && updated_text.chars().last().unwrap().is_ascii() {
                let new_text = updated_text.split_at(processed_text.len()).1.to_string();
                prev_index = current_index;
                current_index = tokens.len();
                
                print!("{new_text}");
                use std::io::Write;
                std::io::stdout().flush()?;
            }
        }
        println!(
            "\n{sample_len} tokens generated ({:.2} token/s)",
            sample_len as f64 / start.elapsed().as_secs_f64(),
        );
        Ok(())
    }
}
