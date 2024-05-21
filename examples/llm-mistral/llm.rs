use candle_core::{DType, Device, Tensor};
use candle_transformers::{
    generation::LogitsProcessor,
    models::quantized_mistral::{Config as QMistralCfg, Model as QMistral},
    quantized_var_builder::VarBuilder,
    utils::apply_repeat_penalty,
};
use hf_hub::{api::sync::Api, Repo};
use prest::*;
use tokenizers::Tokenizer;

pub fn init() -> Result<Mistral> {
    init_with_opts(Default::default())
}

pub fn init_with_opts(cfg: MistralConfig) -> Result<Mistral> {
    let start = std::time::Instant::now();
    info!("started initializing the model...");
    let repo = Repo::model("lmz/candle-mistral".to_owned());
    let repo_api = Api::new().unwrap().repo(repo);
    let tokenizer_filename = repo_api.get("tokenizer.json").unwrap();
    let tokenizer = Tokenizer::from_file(tokenizer_filename).unwrap();
    let eos_token = *tokenizer.get_vocab(true).get("</s>").unwrap();
    let logits_processor = LogitsProcessor::new(cfg.seed, cfg.temperature, cfg.top_p);
    let weights_filename = repo_api.get("model-q4k.gguf")?;
    let mistral_cfg = QMistralCfg::config_7b_v0_1(true);
    let weights = VarBuilder::from_gguf(&weights_filename)?;
    let model = QMistral::new(&mistral_cfg, weights)?;
    info!("initialized the model in {:?}", start.elapsed());
    Ok(Mistral {
        model,
        logits_processor,
        cfg,
        tokenizer,
        eos_token,
        history: String::new(),
        tokens: vec![],
        current_ctx: 0,
        processed: 0,
    })
}

pub struct MistralConfig {
    pub seed: u64,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
}
impl Default for MistralConfig {
    fn default() -> Self {
        Self {
            seed: 123456789,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            temperature: None,
            top_p: None,
        }
    }
}

pub struct Mistral {
    model: QMistral,
    logits_processor: LogitsProcessor,
    tokenizer: Tokenizer,
    cfg: MistralConfig,
    pub history: String,
    tokens: Vec<u32>,
    eos_token: u32,
    pub current_ctx: usize,
    processed: usize,
}

impl Mistral {
    pub fn prompt(&mut self, text: &str) -> Result {
        self.history += text;
        self.tokens.append(&mut self.encode(text)?);
        self.processed = self.tokens.len();
        Ok(())
    }
    pub fn more(&mut self) -> bool {
        let next_token = self.predict().unwrap();
        self.current_ctx = self.tokens.len();
        self.tokens.push(next_token);
        self.try_decode();
        return next_token != self.eos_token;
    }
    fn predict(&mut self) -> Result<u32> {
        let Mistral {
            tokens,
            current_ctx,
            cfg,
            ..
        } = self;
        let input = Tensor::new(&tokens[*current_ctx..], &Device::Cpu)?.unsqueeze(0)?;
        let logits = self.model.forward(&input, *current_ctx)?;
        let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
        let penalty_pos = tokens.len().saturating_sub(cfg.repeat_last_n);
        let logits = apply_repeat_penalty(&logits, cfg.repeat_penalty, &tokens[penalty_pos..])?;
        let next_token = self.logits_processor.sample(&logits)?;
        Ok(next_token)
    }
    fn encode(&self, input: &str) -> Result<Vec<u32>> {
        Ok(self
            .tokenizer
            .encode(input, true)
            .map_err(Error::msg)?
            .get_ids()
            .to_vec())
    }
    fn try_decode(&mut self) {
        let Mistral {
            tokens,
            processed,
            current_ctx,
            ..
        } = self;
        let processed_text = self
            .tokenizer
            .decode(&tokens[*processed..*current_ctx], true)
            .unwrap();
        let updated_text = self.tokenizer.decode(&tokens[*processed..], true).unwrap();
        if updated_text.len() > processed_text.len()
            && updated_text.chars().last().unwrap().is_ascii()
        {
            self.processed = self.current_ctx;
            let new_text = updated_text.split_at(processed_text.len()).1.to_string();
            self.history += &new_text;
        }
    }
}
