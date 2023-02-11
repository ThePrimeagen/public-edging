/**
 * ------------------ WARNING ---------------------
 * Lots of clones and things to just get the job done first.
 *
 *
 * If you get stuck on that point, you are big dumb, focus on solving the
 * problem first plz
 */








use anyhow::{Context, Result};
use std::{collections::HashMap, path::{PathBuf, Path}};
use walkdir::WalkDir;

fn walk(mut dir: PathBuf) -> Result<Vec<PathBuf>> {
    return Ok(WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect());
}

#[derive(Debug)]
struct Templates {
    route: String,
    page: String,
}

const ROUTE_TEMPLATE_PATH: &str = "./route.rs.template";
const MOD_TEMPLATE_PATH: &str = "./pages.mod.rs.template";
const ROUTE_PATH: &str = "./src/router.rs";
const MOD_PATH: &str = "./src/pages/mod.rs";

fn get_template() -> Result<Templates> {
    return Ok(Templates {
        route: std::fs::read_to_string(ROUTE_TEMPLATE_PATH)
            .context("unable to read the route template")?,
        page: std::fs::read_to_string(MOD_TEMPLATE_PATH)
            .context("unable to read the pages.mod template")?,
    });
}

#[derive(Debug)]
struct Page {
	file_name: String,
	file_name_wo_ext: Option<String>,
	project_path: PathBuf,
	project_parent_path: PathBuf,
	page_path: String,
	dir: bool,
	is_mod: bool,
}

fn describe(dir: &Path, file: &Path) -> Result<Page> {
	let metadata = file.metadata()?;
	let file = file.strip_prefix(&dir)?;

	let mut project_base_path = PathBuf::from("src");
	project_base_path.push("pages");

	let project_path = project_base_path.join(file.clone());
	let project_parent_path = project_path.parent().unwrap().to_path_buf();

	let mut file_without_ext = None;
	if metadata.is_file() {
		file_without_ext = Some(file.with_extension("").to_string_lossy().to_string());
	}

	return Ok(Page {
		file_name: file.to_string_lossy().to_string(),
		file_name_wo_ext: file_without_ext,
		project_path,
		project_parent_path,
		page_path: file.to_string_lossy().to_string(),
		dir: metadata.is_dir(),
		is_mod: file.ends_with("mod.rs"),
	});
}

fn main() -> Result<()> {
	let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

	let dir = PathBuf::from(dir);
	let dir = dir.join("src").join("pages");

	let template = get_template()?;

	let mut mod_files: HashMap<PathBuf, Vec<String>> = HashMap::new();
	let mut routes: HashMap<String, String> = HashMap::new();

	let all_paths = walk(dir.clone())?
		.iter()
		.map(|x| describe(&dir, x).expect("unable to describe path"))
		.filter(|x| !x.is_mod) // drop all mod files, they will be modified
		.collect::<Vec<Page>>();

	// Pass 1, build the mod tree
	for page in all_paths.iter().filter(|x| x.dir) {
		mod_files.insert(page.project_path.clone(), vec![]);

		let top = PathBuf::from("src/pages");
        let mut parent = page.project_path.parent();
		let mut curr = Some(page.project_path.as_path());
		while let Some(c) = curr {
            if let Some(p) = parent {
                if p != PathBuf::from("src") {
                    let mut mod_name = c
                        .to_string_lossy()
                        .to_string()
                        .replace(std::path::MAIN_SEPARATOR, "::");

                    mod_name = mod_name.replace("src::pages::", "");
                    if mod_name.ends_with("::") {
                        mod_name = mod_name[0..mod_name.len() - 2].to_string();
                    }

                    let vec = mod_files
                        .entry(p.to_path_buf().clone())
                        .or_insert(vec![]);
                    if !vec.contains(&mod_name) {
                        vec.push(mod_name);
                    }
                }
            }

            mod_files.entry(c.to_path_buf().clone()).or_insert(vec![]);

            // i know... this sucks
            if c == top {
                break;
            }

			curr = c.parent();
			parent = parent.and_then(|x| x.parent());
		}

		/*
		let mod_parent = project_path.parent().unwrap().to_string_lossy().to_string();
		println!("cargo:warning=    adding mod: mod_parent={:?} ", mod_parent);
		if mod_parent == "src" {
			continue;
		}

		println!("cargo:warning=adding directory for mod {:?} ", mod_parent);

		mod_files
			.entry(project_path.to_string_lossy().to_string())
			.or_insert(vec![]);

		mod_files
			.entry(mod_parent)
			.or_insert(vec![])
			.push(project_path.to_string_lossy().to_string().replace(std::path::MAIN_SEPARATOR, "::"));
			*/

	}

    println!("cargo:warning=mod_files: {:?} ", mod_files);

    mod_files
        .iter()
        .map(|(k, v)| {
            let v = v.iter().map(|x| format!("pub mod {};", x)).collect::<Vec<String>>();
            return (k, v)
        })
        .for_each(|(path, m)| {
            let mods = m.join("\n");
            let mods = template.page.replace("__MODS__", &mods);
            println!("cargo:warning=mod file contents: {:?} ", mods);

            if let Err(e) = std::fs::write(path.join("mod.rs"), mods) {
                panic!("unable to write mod file: {:?}", e);
            }
        });

	todo!("finish");
	/*
	/*
	let file_name = page_path.file_name().unwrap().to_string_lossy().to_string();

	if file_name_and_path.ends_with("mod") {
	println!("cargo:warning=skipping mod file {:?} ", file_name_and_path);
	continue;
	}

	println!("cargo:warning={:?} ", file_name_and_path);
	if !file_name.ends_with(".rs") {
	continue;
	}

	let route_crate_path = file_name_and_path.replace(std::path::MAIN_SEPARATOR, "::");
	routes.insert(file_name_and_path, route_crate_path);

	println!("cargo:warning=project_path{:?} ", project_path);
	mod_files
	.get_mut(&project_path.to_string_lossy().to_string())
	.unwrap() // currently assuming this should "always" exist
	.push(file_name[..file_name.len() - 3].to_string());
	*/
    }

    println!("cargo:warning=routes {:?} ", routes);
    println!("cargo:warning=mod_files {:?} ", mod_files);

    let routes = routes
        .iter()
		.map(|(k, v)| {
			if k.ends_with("index") {
				return (k[..k.len() - 5].to_string(), v.clone());
			}
			return (k.clone(), v.clone());
		})
        .map(|(route, route_call)| format!("        .get_async(\"/{}\", |_, ctx| async move {{
            return {}::route(ctx).await;
        }})", route, route_call))
        .collect::<Vec<String>>();

    let routes = routes.join("\n");
    let routes = template.route.replace("__ROUTES__", &routes);
    std::fs::write(ROUTE_PATH, routes)?;

	/*
	*/

    return Ok(());
	*/
}
