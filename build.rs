use anyhow::Result;
use syn::Visibility;
use std::path::{Path, PathBuf};

/**
 * ------------------ WARNING ---------------------
 * Lots of clones and things to just get the job done first.
 *
 *
 * If you get stuck on that point, you are big dumb, focus on solving the
 * problem first plz
 */

const ROUTE_TEMPLATE_PATH: &str = "./route.rs.template";
const ROUTE_PATH: &str = "router/src/lib.rs";

/**
fn walk(mut dir: PathBuf) -> Result<Vec<PathBuf>> {
    return Ok(WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect());
}
*/

#[derive(Debug)]
struct Pages {
    librs: PathBuf,
}

fn list_routes(dir: &Path) -> Result<Vec<String>> {
    todo!("Implement this you piece");
}

#[derive(Debug)]
struct Page {
    route: PathBuf,
    mods: Vec<PathBuf>,
    routes: Vec<&'static str>,
}

// pub mod foo;
// is foo a file?
// is foo a directory?
//
// ./foo.rs
fn describe(f: &Path) -> Result<Page> {
    let route = f.parent().unwrap().to_path_buf(); // DANGEROUS

    let file = std::fs::read_to_string(f)?;
    let file = syn::parse_file(file.as_str())?;

    let mut files = vec![];
    let mut routes = vec![];
    for item in file.items {
        if let syn::Item::Mod(item) = &item {
            if let Visibility::Public(_) = item.vis {
                files.push(item.ident.to_string());
            }
        }

        if let syn::Item::Fn(item) = &item {
            if let Visibility::Public(_) = item.vis {
                if item.sig.ident.to_string() == "get" {
                    routes.push("get");
                }
            }
        }
    }

    let mut mods = vec![];
    for file in files {
        // TODO: Sorry rustaceans, this is a hack that is
        let mut path = f.to_path_buf().parent().unwrap().to_path_buf();
        path.push(file);

        if path.is_dir() {
            mods.push(path);
            continue;
        }

        // grows
        let path = path.to_str().unwrap().to_string();
        let path = format!("{}.rs", path);
        let path = PathBuf::from(path);

        if path.is_file() {
            mods.push(path);
        } else {
            // WHAT THE HELL IS EVEN THAT?
            unreachable!("This shouldn't happen? {:?}", path);
        }
    }

    return Ok(Page {
        route,
        mods,
        routes,
    });
}

fn main() -> Result<()> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut base = PathBuf::from(dir);
    base.push("pages");
    base.push("src");

    let mut librs = PathBuf::from(&base);
    librs.push("lib.rs");

    let mut files = vec![librs];
    let mut all_describes = vec![];

    while let Some(mut file) = files.pop() {
        if file.is_dir() {
            file.push("mod.rs");
        }

        let mod_files = describe(&file)?;

        files.extend(mod_files.mods.clone().into_iter());

        all_describes.push(mod_files);
    }

    let mut routes = vec![];
    for describe in all_describes {
        if describe.routes.is_empty() {
            continue;
        }

        let base_route = describe.route.strip_prefix(&base).unwrap();
        let mut base_route = base_route.to_str().unwrap().to_string();

        if base_route.ends_with("/") {
            base_route.pop();
        }

        if !base_route.starts_with("/") {
            base_route = format!("/{}", base_route);
        }

        /*
        for route in describe.routes {
            if base_route.ends_with("::") {
                println!("cargo:warning=route={}", format!("{}{}", base_route, route));
            } else {
                println!("cargo:warning=route={}", format!("{}::{}", base_route, route));
            }
        }
        */

        println!("cargo:warning=route={}", base_route);

        let mut base_path = base_route.clone();
        base_path.replace_range(0..1, "pages::");
        let base_path = base_path.replace("/", "::");

        for route in describe.routes {
            routes.push(format!("        .{}_async(\"{}\", move |ctx| {{
            return {}::{}(ctx);
        }})", route, base_route, base_path, route).replace("::::", "::"));
        }
    }


    let router_template = std::fs::read_to_string(ROUTE_TEMPLATE_PATH)?;
    let router_template = router_template.replace("__ROUTES__", &routes.join("\n"));

    std::fs::write(ROUTE_PATH, router_template)?;

    return Ok(());
}
