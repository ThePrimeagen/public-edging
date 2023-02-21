use anyhow::{Result, anyhow};
use syn::Visibility;
use std::{path::{Path, PathBuf}, convert::{TryFrom, TryInto}, fmt::Display};

#[derive(Debug)]
enum Method {
    Get,
    Post
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "get"),
            Method::Post => write!(f, "post"),
        }
    }
}

#[derive(Debug)]
struct PageDescription {
    path: PathBuf,
    mods: Vec<String>,
    routes: Vec<Method>,
}

impl TryFrom<&Path> for PageDescription {

    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let file = std::fs::read_to_string(value)?;
        let file = syn::parse_file(file.as_str())?;

        let mut mods = vec![];
        let mut routes = vec![];

        for item in file.items {
            if let syn::Item::Mod(item) = &item {
                if let Visibility::Public(_) = item.vis {
                    mods.push(item.ident.to_string());
                }
            }

            if let syn::Item::Fn(item) = &item {
                if let Visibility::Public(_) = item.vis {
                    if item.sig.ident.to_string() == "get" {
                        routes.push(Method::Get);
                    }
                }
            }
        }

        return Ok(PageDescription {
            path: value.to_path_buf(),
            mods,
            routes,
        });
    }
}


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

#[derive(Debug)]
struct Pages {
    librs: PathBuf,
}

#[derive(Debug)]
struct Page {
    path: PathBuf,
    mods: Vec<PathBuf>,
    routes: Vec<Method>,
}

fn get_potential_filenames(base_path: &PathBuf, mod_ident: &String) -> (PathBuf, PathBuf) {
    // TODO: Sorry rustaceans, this is a hack that is
    let possible_file_name = format!("{}.rs", mod_ident);

    let mut file_path = base_path.clone();
    file_path.push(possible_file_name);

    let mut mod_path = base_path.clone();
    mod_path.push(mod_ident);
    mod_path.push("mod.rs");

    return (file_path, mod_path);
}

impl TryFrom<&Path> for Page {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {

        let desc: PageDescription = value.try_into()?;
        let base_path = value.parent().unwrap().to_path_buf();
        let mut mod_paths = vec![];

        for file in desc.mods {
            let (file_path, mod_path) = get_potential_filenames(&base_path, &file);

            // We don't allow for this situation right now.
            // This situation is confusing
            if file_path.is_file() && mod_path.is_file() {
                return Err(anyhow!(format!(
                    "You cannot have a file and directory/mod.rs with the same name: {:?} and {:?} cannot exist",
                    file_path, mod_path
                )));
            }

            println!("base: {:?}, file: {:?}", base_path, file);
            if !file_path.is_file() && !mod_path.is_file() {
                return Err(anyhow!(format!(
                    "Somehow path {:?} was detected but neither {:?} nor {:?} exists.",
                    file, file_path, mod_path
                )));
            }

            if file_path.is_file() {
                mod_paths.push(file_path);
            } else {
                mod_paths.push(mod_path);
            }
        }

        let page = Page {
            path: value.to_path_buf(),
            mods: mod_paths,
            routes: desc.routes,
        };

        println!("page: {:?}", page);

        return Ok(page);
    }
}


fn main() -> Result<()> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut base = PathBuf::from(dir);
    base.push("pages");
    base.push("src");

    let mut librs = PathBuf::from(&base);
    librs.push("lib.rs");

    let mut files = vec![librs];
    let mut pages = vec![];

    while let Some(file) = files.pop() {
        let page: Page = file.as_path().try_into()?;

        files.extend(page.mods.clone().into_iter());

        pages.push(page);
    }

    let mut routes = vec![];
    for page in pages {
        if page.routes.is_empty() {
            continue;
        }

        let base_route = page.path.strip_prefix(&base).unwrap();
        let mut base_route = base_route.to_str().unwrap().to_string();

        if base_route.ends_with("/") {
            base_route.pop();
        }

        if !base_route.starts_with("/") {
            base_route = format!("/{}", base_route);
        }

        println!("cargo:warning=route={}", base_route);

        // i am ashamed
        if base_route.ends_with("lib.rs") {
            base_route = base_route[..base_route.len() - "lib.rs".len()].to_string();
        }
        if base_route.ends_with("mod.rs") {
            base_route = base_route[..base_route.len() - "mod.rs".len()].to_string();
        }
        if base_route.ends_with(".rs") {
            base_route = base_route[..base_route.len() - ".rs".len()].to_string();
        }

        let mut base_path = base_route.clone();
        base_path.replace_range(0..1, "pages::");
        let base_path = base_path.replace("/", "::");

        for route in page.routes {
            routes.push(format!("        .{}_async(\"{}\", move |ctx| {{
            return {}::{}(ctx);
        }})", route, base_route, base_path, route).replace("::::", "::"));
        }
    }


    /*
    let router_template = std::fs::read_to_string(ROUTE_TEMPLATE_PATH)?;
    let router_template = router_template.replace("__ROUTES__", &routes.join("\n"));

    std::fs::write(ROUTE_PATH, router_template)?;
    */

    return Ok(());
}
