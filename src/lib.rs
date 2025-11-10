use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use swc_core::common::{
    DUMMY_SP,
    SyntaxContext
};
use swc_core::ecma::{
    ast::*,
    visit::{FoldWith, Fold},
};
use swc_core::plugin::{
    plugin_transform,
    proxies::TransformPluginProgramMetadata,
    metadata::TransformPluginMetadataContextKind
};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub root_dir: Option<String>,
}

pub struct RawImport {
    root: PathBuf,
    cwd: PathBuf,
}

impl RawImport {
    pub fn new(root_dir: String, current_path: String) -> Self {
        let mut cwd = PathBuf::from("/cwd");
        let current = current_path.replace(&root_dir, "");

        // 移除开头的斜杠
        let trimmed_current = current.trim_start_matches('/');
        cwd = cwd.join(trimmed_current);
        
        // 获取文件所在目录
        if cwd.is_file() {
            cwd.pop();
        }

        RawImport {
            root: PathBuf::from("/cwd"),
            cwd,
        }
    }

    fn read_file(&mut self, raw_path: String) -> String {
        let is_relative = raw_path.starts_with('.');
        let path: PathBuf = if is_relative {
            self.cwd.join(PathBuf::from(&raw_path))
        } else {
            self.root
                .join("node_modules")
                .join(&raw_path)
        };
        let names = normalize_path(path.display().to_string().replace("\0", ""));

        match fs::read_to_string(&names) {
            Ok(s) => s,
            Err(err) => panic!("Failed to read {}: {}", names, err),
        }
    }
}

impl Fold for RawImport {
    fn fold_module_items(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        let mut new_items = Vec::with_capacity(items.len());
        
        for item in items {
            match item {
                // 处理带有?raw的导入声明
                ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
                    let src_str = import_decl.src.value.as_str().unwrap_or_default().to_string();
                    
                    if let Some((path, _)) = src_str.split_once("?raw") {
                        // 处理每个导入说明符
                        for specifier in import_decl.specifiers {
                            if let ImportSpecifier::Default(default_spec) = specifier {
                                // 读取文件内容
                                let content = self.read_file(path.to_string());
                                
                                // 创建const声明
                                let var_decl = VarDecl {
                                    span: DUMMY_SP,
                                    kind: VarDeclKind::Const,
                                    ctxt: SyntaxContext::empty(),
                                    decls: vec![VarDeclarator {
                                        span: DUMMY_SP,
                                        name: Pat::Ident(default_spec.local.clone().into()),
                                        init: Some(Box::new(Expr::Lit(Lit::Str(Str {
                                            span: DUMMY_SP,
                                            value: content.into(),
                                            raw: None,
                                        })))),
                                        definite: false,
                                    }],
                                    declare: false,
                                };
                                
                                new_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(var_decl)))));
                            } else {
                                // 非默认导入抛出错误（根据需求调整）
                                panic!("Only default imports are supported for ?raw");
                            }
                        }
                    } else {
                        // 保留普通导入
                        new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
                    }
                }
                // 保留其他类型的模块项
                _ => new_items.push(item),
            }
        }
        
        new_items
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: Config = serde_json::from_str(
        &metadata.get_transform_plugin_config()
            .expect("Should provide plugin config")
    ).unwrap();

    let root_dir = config
        .root_dir
        .expect("Should provide `rootDir` in plugin config");
    let current_path = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap_or_else(|| root_dir.clone());
    
    program.fold_with(&mut RawImport::new(root_dir, current_path.to_string()))
}
// rust格式化路径
fn normalize_path(path: String) -> String {
    let mut parts: Vec<&str> = Vec::new();
    let is_absolute = path.starts_with('/');
    let starts_with_dot = path.starts_with("./");
    
    // Split path by '/' and filter out empty parts and '.'
    for part in path.split('/') {
        match part {
            "" | "." => continue, // Skip empty parts and current directory
            ".." => {
                // Go up one directory level
                if !parts.is_empty() && parts.last() != Some(&"..") {
                    parts.pop();
                } else if !is_absolute {
                    // Only add ".." for relative paths when we can't go further up
                    parts.push("..");
                }
            }
            _ => parts.push(part),
        }
    }
    
    // Reconstruct the path
    let mut result = if is_absolute {
        "/".to_string()
    } else if starts_with_dot && !parts.is_empty() {
        "./".to_string()
    } else if parts.is_empty() && starts_with_dot {
        ".".to_string()
    } else {
        String::new()
    };
    
    // Join the parts
    if !parts.is_empty() {
        if is_absolute {
            result.push_str(&parts.join("/"));
        } else if starts_with_dot {
            result.push_str(&parts.join("/"));
        } else {
            result = parts.join("/");
        }
    } else if is_absolute {
        // Keep just "/" for absolute root
        result = "/".to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        // Test cases from the examples
        assert_eq!(
            normalize_path("/workspaces/raw-import//./package.json".to_string()),
            "/workspaces/raw-import/package.json"
        );
        
        assert_eq!(
            normalize_path("/workspaces/raw-import/..//./package.json".to_string()),
            "/workspaces/package.json"
        );
        
        assert_eq!(
            normalize_path("./A/../package.json".to_string()),
            "./package.json"
        );
        assert_eq!(
            normalize_path("/cwd//./package.json".to_string()),
            "/cwd/package.json"
        );
        // Additional test cases
        assert_eq!(
            normalize_path("/a/b/c/../d".to_string()),
            "/a/b/d"
        );
        
        assert_eq!(
            normalize_path("../a/b".to_string()),
            "../a/b"
        );
        
        assert_eq!(
            normalize_path("./a/./b/../c".to_string()),
            "./a/c"
        );
        
        assert_eq!(
            normalize_path("/".to_string()),
            "/"
        );
        
    }
}
