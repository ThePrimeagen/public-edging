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
	project_path: String,
	project_parent_path: Option<String>,
	page_path: String,
	dir: bool,
}

fn describe(dir: &Path, file: &Path) -> Result<Option<Page>> {
	let file = file.strip_prefix(&dir)?;

	let mut project_path = PathBuf::from("src");
	project_path.push("pages");

	let mut file_parent = Some(PathBuf::from(""));
	if let Some(parent) = file.parent() {
		file_parent = Some(parent.to_path_buf());
	}
	project_path.push(file_parent.unwrap());

	let metadata = project_path.metadata()?;
	let mut file_without_ext = None;
	if metadata.is_file() {
		file_without_ext = Some(file.with_extension("").to_string_lossy().to_string());
	}

	if let Some(f) = file_without_ext.clone() {
		project_path.push(f);
	} else {
		project_path.push(file.clone());
	}

	let mut project_parent_path = None;
	if let Some(parent) = project_path.parent() {
		project_parent_path = Some(parent.to_string_lossy().to_string());
	}

	return Ok(Some(Page {
		file_name: file.to_string_lossy().to_string(),
		file_name_wo_ext: file_without_ext,
		project_path: project_path.to_string_lossy().to_string(),
		project_parent_path,
		page_path: file.to_string_lossy().to_string(),
		dir: metadata.is_dir(),
	}));
}

fn main() -> Result<()> {
	let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

	let dir = PathBuf::from(dir);
	let dir = dir.join("src").join("pages");

	let template = get_template()?;

	let mut mod_files: HashMap<String, Vec<String>> = HashMap::new();
	let mut routes: HashMap<String, String> = HashMap::new();

	let all_paths = walk(dir.clone())?;
	let dirs = all_paths
		.iter()
		.filter_map(|x| x.metadata().ok().map(|m| (x, m)))
		.filter(|(_, m)| m.is_dir())
		.map(|(x, _)| x)
		.collect::<Vec<_>>();

	for path in dirs {
		println!("cargo:warning=dir {:?}", path);
		let page = describe(&dir, path)?;
		if let None = page {
			continue;
		}

		println!("cargo:warning=page {:?} ", page);
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

	println!("cargo:warning=mod_files {:?} ", mod_files);
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
    let mods = mods
        .iter()
        .map(|x| format!("pub mod {};", x))
        .collect::<Vec<String>>();
    let mods = mods.join("\n");
    let mods = template.page.replace("__MODS__", &mods);

    std::fs::write(MOD_PATH, mods)?;
	*/

    return Ok(());
	*/
}
