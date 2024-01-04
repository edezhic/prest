use anyhow::Error;
use prest_build::out_path;
use std::{collections::HashMap, sync::Arc};

fn main() {
    let contents = swc_run("./script.ts", false, false).unwrap();
    std::fs::write(out_path("script.js"), contents).unwrap();
}

use swc_bundler::{Bundler, Hook, Load, ModuleData, ModuleRecord};
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, Globals, Mark, SourceMap, Span, GLOBALS,
};
use swc_ecma_ast::*;
use swc_ecma_codegen::{
    text_writer::{omit_trailing_semi, JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_loader::{
    resolvers::{lru::CachingResolver, node::NodeModulesResolver},
    TargetEnv,
};
use swc_ecma_parser::{parse_file_as_module, Syntax, TsConfig};
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_visit::{FoldWith, VisitMutWith};

fn swc_run(main: &str, minify: bool, tree_shaking: bool) -> Result<String, Error> {
    // starting points for each bundle's build
    let mut entries = HashMap::default();
    entries.insert("main".into(), swc_common::FileName::Real(main.into()));
    let source_map_rc = Arc::new(SourceMap::default());

    // bundler helpers to handle imports
    let loader = Loader(source_map_rc.clone());
    let resolver = CachingResolver::new(
        4096,
        NodeModulesResolver::new(TargetEnv::Browser, Default::default(), true),
    );

    // dummy hook for import.meta props
    let hook = Box::new(ImportMetaProps);

    let bundler_config = swc_bundler::Config {
        require: false,
        disable_inliner: true,
        external_modules: Default::default(),
        disable_fixer: minify,
        disable_hygiene: minify,
        disable_dce: !tree_shaking,
        module: Default::default(),
    };

    use swc_ecma_minifier::option::{
        CompressOptions, MangleOptions, MinifyOptions, TopLevelOptions,
    };
    let minify_options = &MinifyOptions {
        compress: Some(CompressOptions {
            top_level: Some(TopLevelOptions { functions: true }),
            ..Default::default()
        }),
        mangle: Some(MangleOptions {
            top_level: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };

    let code = GLOBALS.set(&Globals::new(), || {
        let globals = Box::leak(Box::new(Globals::default()));

        let mut bundler = Bundler::new(
            globals,
            source_map_rc.clone(),
            loader,
            resolver,
            bundler_config,
            hook,
        );

        let mut bundles = bundler
            .bundle(entries)
            .map_err(|err| println!("{:?}", err))
            .expect("should bundle stuff");

        if minify {
            let minify_extra_options = &swc_ecma_minifier::option::ExtraOptions {
                unresolved_mark: Mark::new(),
                top_level_mark: Mark::new(),
            };
            bundles = bundles
                .into_iter()
                .map(|mut b| {
                    GLOBALS.set(globals, || {
                        b.module = swc_ecma_minifier::optimize(
                            b.module.into(),
                            source_map_rc.clone(),
                            None,
                            None,
                            &minify_options,
                            &minify_extra_options,
                        )
                        .expect_module();
                        b.module.visit_mut_with(&mut fixer(None));
                        b
                    })
                })
                .collect();
        }
        // if no minification - at least strip out typescript notations
        bundles = bundles
            .into_iter()
            .map(|mut b| {
                GLOBALS.set(globals, || {
                    b.module = Into::<Program>::into(b.module)
                        .fold_with(&mut swc_ecma_transforms_typescript::strip(Mark::new()))
                        .expect_module();
                    b.module.visit_mut_with(&mut fixer(None));
                    b
                })
            })
            .collect();

        // since we're building only 1 bundle
        let bundled = &bundles[0];

        // write it's emitted pieces into buffer
        let mut buf = vec![];
        {
            let writer = JsWriter::new(source_map_rc.clone(), "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: swc_ecma_codegen::Config::default().with_minify(minify),
                cm: source_map_rc.clone(),
                comments: None,
                wr: if minify {
                    Box::new(omit_trailing_semi(writer)) as Box<dyn WriteJs>
                } else {
                    Box::new(writer) as Box<dyn WriteJs>
                },
            };

            emitter.emit_module(&bundled.module).unwrap();
        }

        // convert buffer into UTF-8 and save into the file
        let code = String::from_utf8_lossy(&buf).to_string();

        code
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

pub struct Loader(Arc<SourceMap>);
impl Load for Loader {
    fn load(&self, f: &FileName) -> Result<ModuleData, Error> {
        let fm = match f {
            FileName::Real(path) => self.0.load_file(path)?,
            _ => unreachable!(),
        };

        let module = parse_file_as_module(
            &fm,
            Syntax::Typescript(TsConfig::default()),
            EsVersion::Es2020,
            None,
            &mut vec![],
        )
        .unwrap_or_else(|err| {
            let handler =
                Handler::with_tty_emitter(ColorConfig::Always, true, false, Some(self.0.clone()));
            err.into_diagnostic(&handler).emit();
            panic!(
                "failed to parse(load) file {}",
                match f {
                    FileName::Real(path) => path.to_str().unwrap(),
                    _ => unreachable!(),
                }
            )
        });

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}
