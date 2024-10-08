use std::path::{Path, PathBuf};

use glob::Pattern;
use glob_match::glob_match;

use crate::module::{relative_to_root, ModuleInfo};
use crate::resolve::{ResolvedResource, ResolverResource};

impl ModuleInfo {
    pub fn described_side_effect(&self) -> Option<bool> {
        if let Some(ResolverResource::Resolved(ResolvedResource(source))) = &self.resolved_resource
        {
            match &source.package_json() {
                Some(desc) => {
                    let value = desc.raw_json();
                    let side_effects = value.get("sideEffects".to_string());

                    let root: &Path = desc.directory();
                    let root: PathBuf = root.into();

                    side_effects.map(|side_effect| {
                        Self::match_flag(
                            side_effect,
                            relative_to_root(&self.file.path.to_string_lossy().to_string(), &root)
                                .as_str(),
                        )
                    })
                }
                None => None,
            }
        } else {
            None
        }
    }

    /**
     * 获取当前的模块是否具备 sideEffects
     */
    #[allow(dead_code)]
    pub fn get_side_effects_flag(&self) -> bool {
        if let Some(ResolverResource::Resolved(ResolvedResource(source))) = &self.resolved_resource
        {
            match &source.package_json() {
                Some(desc) => {
                    let value = desc.raw_json();
                    let side_effects = value.get("sideEffects".to_string());

                    match side_effects {
                        Some(side_effect) => {
                            let root: &Path = desc.directory();
                            let root: PathBuf = root.into();

                            Self::match_flag(
                                side_effect,
                                relative_to_root(
                                    &self.file.path.to_string_lossy().to_string(),
                                    &root,
                                )
                                .as_str(),
                            )
                        }
                        None => true,
                    }
                }
                None => true,
            }
        } else {
            true
        }
    }
    fn match_flag(flag: &serde_json::Value, path: &str) -> bool {
        match flag {
            // NOTE: 口径需要对齐这里：https://github.com/webpack/webpack/blob/main/lib/optimize/SideEffectsFlagPlugin.js#L331
            serde_json::Value::Bool(flag) => *flag,
            serde_json::Value::String(flag) => match_glob_pattern(flag, path),
            serde_json::Value::Array(flags) => {
                flags.iter().any(|flag| Self::match_flag(flag, path))
            }
            _ => true,
        }
    }
}

fn match_glob_pattern(pattern: &str, path: &str) -> bool {
    let trimmed = path.trim_start_matches("./");

    // TODO: cache
    if !pattern.contains('/') {
        return Pattern::new(format!("**/{}", pattern).as_str())
            .unwrap()
            .matches(trimmed);
    }

    glob_match(pattern.trim_start_matches("./"), trimmed)
}

#[cfg(test)]
mod tests {
    use super::match_glob_pattern;
    use crate::utils::test_helper::{get_module, setup_compiler};

    #[test]
    fn test_path_side_effects_no_dot_start_pattern() {
        assert!(match_glob_pattern("esm/index.js", "./esm/index.js",));
    }

    #[test]
    fn test_exact_path_side_effects_flag() {
        assert!(match_glob_pattern("./src/index.js", "./src/index.js",));
    }

    #[test]
    fn test_exact_path_side_effects_flag_negative() {
        assert!(!match_glob_pattern("./src/index.js", "./dist/index.js",));
    }

    #[test]
    fn test_wild_effects_flag() {
        assert!(match_glob_pattern(
            "./src/lib/**/*.s.js",
            "./src/lib/apple/pie/index.s.js",
        ));
    }

    #[test]
    fn test_double_wild_starts_effects_flag() {
        assert!(match_glob_pattern(
            "**/index.js",
            "./deep/lib/file/index.js",
        ));
    }

    #[test]
    fn test_side_effects_flag() {
        let compiler = setup_compiler("test/build/side-effects-flag", false);
        compiler.compile().unwrap();
        let foo = get_module(&compiler, "node_modules/foo/index.ts");
        let bar = get_module(&compiler, "node_modules/bar/index.ts");
        let zzz = get_module(&compiler, "node_modules/zzz/index.ts");
        let four = get_module(&compiler, "node_modules/four/index.ts");
        let four_s = get_module(&compiler, "node_modules/four/index.s.ts");
        assert!(!foo.info.unwrap().get_side_effects_flag());
        assert!(bar.info.unwrap().get_side_effects_flag());
        assert!(zzz.info.unwrap().get_side_effects_flag());
        assert!(!four.info.unwrap().get_side_effects_flag());
        assert!(four_s.info.unwrap().get_side_effects_flag());
    }
}
