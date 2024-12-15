use anyhow::{Error, Result};
use std::{collections::HashMap, sync::Arc};
use swc_bundler::{Bundler, Hook, Load, ModuleData, ModuleRecord};
use swc_common::{
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    FileName, Mark, SourceMap, Span, GLOBALS,
};
use swc_ecma_ast::*;
use swc_ecma_codegen::{
    text_writer::{JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_loader::{
    resolvers::{lru::CachingResolver, node::NodeModulesResolver},
    TargetEnv,
};
use swc_ecma_minifier::option::{CompressOptions, MangleOptions, MinifyOptions, TopLevelOptions};
use swc_ecma_parser::{parse_file_as_module, EsSyntax, Syntax, TsSyntax};
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_visit::VisitMutWith;

use std::{fs::write, path::Path};

pub fn bundle_ts(path: &str) -> Result<()> {
    // let minify = !cfg!(debug_assertions);
    let minify = false; // TODO: currently breaks things
    let js = swc_run(path, minify)?;
    let ts_filename = Path::new(path).file_name().unwrap().to_str().unwrap();
    let js_filename = ts_filename.replace(".tsx", ".js").replace(".ts", ".js");
    let out_file = super::out_path(&js_filename);
    write(out_file, js)?;
    Ok(())
}

fn swc_run(main: &str, minify: bool) -> Result<String, Error> {
    let globals = Box::leak(Box::default());
    let code = GLOBALS.set(globals, || {
        let mut entries = HashMap::default();
        entries.insert("main".into(), swc_common::FileName::Real(main.into()));
        let source_map_rc = Arc::new(SourceMap::default());

        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        let loader = Loader {
            srcmap: source_map_rc.clone(),
            top_level_mark,
            unresolved_mark,
        };

        let resolver = CachingResolver::new(
            4096,
            NodeModulesResolver::new(TargetEnv::Browser, Default::default(), true),
        );

        let hook = Box::new(ImportMetaProps);

        let bundler_config = swc_bundler::Config {
            disable_dce: true,
            ..Default::default()
        };
        let mut bundler = Bundler::new(
            globals,
            source_map_rc.clone(),
            loader,
            resolver,
            bundler_config,
            hook,
        );
        let mut bundles = bundler.bundle(entries).expect("should bundle stuff");
        let mut bundle = bundles.pop().expect("should produce a bundle");

        if minify {
            let minify_opts = MinifyOptions {
                compress: Some(CompressOptions {
                    top_level: Some(TopLevelOptions { functions: true }),
                    keep_fnames: true,
                    keep_fargs: true,
                    ..Default::default()
                }),
                mangle: Some(MangleOptions {
                    keep_fn_names: true,
                    top_level: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let minify_extra_options = &swc_ecma_minifier::option::ExtraOptions {
                unresolved_mark,
                top_level_mark,
                mangle_name_cache: None,
            };
            bundle.module = swc_ecma_minifier::optimize(
                bundle.module.into(),
                source_map_rc.clone(),
                None,
                None,
                &minify_opts,
                &minify_extra_options,
            )
            .expect_module();
            bundle.module.visit_mut_with(&mut fixer(None));
        }

        module_to_code(&bundle.module, source_map_rc)
    });
    Ok(code)
}

struct ImportMetaProps;
impl Hook for ImportMetaProps {
    fn get_import_meta_props(
        &self,
        _span: Span,
        _module_record: &ModuleRecord,
    ) -> Result<Vec<swc_ecma_ast::KeyValueProp>, anyhow::Error> {
        Ok(vec![])
    }
}

pub struct Loader {
    srcmap: Arc<SourceMap>,
    unresolved_mark: Mark,
    top_level_mark: Mark,
}

impl Load for Loader {
    fn load(&self, f: &FileName) -> Result<ModuleData, Error> {
        let FileName::Real(path) = f else {
            unimplemented!("Only real files can be used")
        };
        let fm = self.srcmap.load_file(path)?;

        let p = path.to_string_lossy();
        let typescript = p.ends_with("ts") || p.ends_with("tsx");
        let jsx = p.ends_with("tsx") || p.ends_with("jsx");

        let syntax = if typescript {
            let mut syntax = TsSyntax::default();
            syntax.tsx = true;
            Syntax::Typescript(syntax)
        } else {
            let mut syntax = EsSyntax::default();
            syntax.jsx = true;
            Syntax::Es(EsSyntax::default())
        };

        let comments = SingleThreadedComments::default();

        let mut module =
            parse_file_as_module(&fm, syntax, EsVersion::Es2020, Some(&comments), &mut vec![])
                .unwrap_or_else(|err| handle_err(err, f, self.srcmap.clone()));

        if typescript {
            let mut program = Into::<Program>::into(module);
            let config = swc_ecma_transforms_typescript::Config {
                // preserve Preact pragma `h` import and maybe others? Idk wtf
                verbatim_module_syntax: true,
                ..Default::default()
            };
            let mut pass =
                swc_ecma_transforms_typescript::typescript(config, Mark::new(), Mark::new());
            program.mutate(&mut pass);
            module = program.expect_module();
        }

        if jsx {
            let mut program = Into::<Program>::into(module);
            let mut pass = swc_ecma_transforms_react::react(
                self.srcmap.clone(),
                Some(&comments),
                Default::default(),
                self.top_level_mark,
                self.unresolved_mark,
            );
            program.mutate(&mut pass);
            module = program.expect_module();
            module.visit_mut_with(&mut fixer(None));
        }

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}

fn module_to_code(module: &Module, srcmap: Arc<SourceMap>) -> String {
    let mut buf = vec![];
    {
        let writer = JsWriter::new(srcmap.clone(), "", &mut buf, None);
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config::default().with_minify(false),
            cm: srcmap,
            comments: None,
            wr: Box::new(writer) as Box<dyn WriteJs>,
        };

        emitter.emit_module(&module).unwrap();
    }
    String::from_utf8_lossy(&buf).to_string()
}

fn handle_err(err: swc_ecma_parser::error::Error, f: &FileName, srcmap: Arc<SourceMap>) -> Module {
    let handler = Handler::with_tty_emitter(ColorConfig::Always, true, false, Some(srcmap));
    err.into_diagnostic(&handler).emit();
    panic!(
        "failed to parse(load) file {}",
        match f {
            FileName::Real(path) => path.to_str().unwrap(),
            _ => unreachable!(),
        }
    )
}
