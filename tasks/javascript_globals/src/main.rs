#![allow(clippy::print_stdout, clippy::print_stderr)]
use lazy_static::lazy_static;
use oxc_tasks_common::agent;
use rustc_hash::FxHashMap;
use serde::Serialize;
mod template;

#[derive(Serialize, Debug)]
pub struct EnvVar<'a> {
    pub name: &'a str,
    pub writeable: bool,
}

#[derive(Serialize, Debug)]
pub struct Env<'a> {
    pub name: &'a str,
    pub vars: Vec<EnvVar<'a>>,
}

#[derive(Serialize)]
pub struct Context<'a> {
    envs: Vec<Env<'a>>,
}

impl<'a> Context<'a> {
    fn new(envs: Vec<Env<'a>>) -> Self {
        Self { envs }
    }
}

fn get_diff(
    current: &FxHashMap<String, bool>,
    prev: &FxHashMap<String, bool>,
) -> FxHashMap<String, bool> {
    let mut retv: FxHashMap<String, bool> = FxHashMap::default();

    for (key, value) in current {
        if !prev.contains_key(key) {
            retv.insert(key.clone(), *value);
        }
    }

    retv
}

lazy_static! {
    static ref NEW_GLOBALS_2017: FxHashMap<String, bool> = {
        return FxHashMap::from_iter([
            (String::from("Atomics"), false),
            (String::from("SharedArrayBuffer"), false),
        ]);
    };
    static ref NEW_GLOBALS_2020: FxHashMap<String, bool> = {
        return FxHashMap::from_iter([
            (String::from("BigInt"), false),
            (String::from("BigInt64Array"), false),
            (String::from("BigUint64Array"), false),
            (String::from("globalThis"), false),
        ]);
    };
    static ref NEW_GLOBALS_2021: FxHashMap<String, bool> = {
        return FxHashMap::from_iter([
            (String::from("AggregateError"), false),
            (String::from("FinalizationRegistry"), false),
            (String::from("WeakRef"), false),
        ]);
    };
}

fn main() {
    // Each global is given a value of true or false.
    // A value of true indicates that the variable may be overwritten.
    // A value of false indicates that the variable should be considered read-only.
    // open globals.json file relative to current file
    // let globals: FxHashMap<String, FxHashMap<String, bool>>;
    let globals: FxHashMap<String, FxHashMap<String, bool>> = match agent()
        .get("https://raw.githubusercontent.com/sindresorhus/globals/main/globals.json")
        .call()
    {
        Ok(mut response) => response.body_mut().read_json().unwrap(),
        Err(e) => {
            panic!("Failed to fetch globals.json: {e}");
        }
    };

    // 19 variables such as Promise, Map, ...
    let new_globals_2015 = get_diff(&globals["es2015"], &globals["es5"]);

    let new_globals_2015_2017 = {
        let mut map = FxHashMap::default();
        map.extend(new_globals_2015.clone());
        map.extend(NEW_GLOBALS_2017.clone());
        map
    };

    let new_globals_2015_2017_2020 = {
        let mut map = new_globals_2015_2017.clone();
        map.extend(NEW_GLOBALS_2020.clone());
        map
    };

    let new_globals_2015_2017_2020_2021 = {
        let mut map = new_globals_2015_2017_2020.clone();
        map.extend(NEW_GLOBALS_2021.clone());
        map
    };

    let envs_preset: Vec<Env> = [
        // Language
        ("builtin", &globals["builtin"]), // oxc uses builtin instead of es5 of ESLint
        ("es6", &new_globals_2015),
        ("es2015", &new_globals_2015),
        ("es2016", &new_globals_2015),
        ("es2017", &new_globals_2015_2017),
        ("es2018", &new_globals_2015_2017),
        ("es2019", &new_globals_2015_2017),
        ("es2020", &new_globals_2015_2017_2020),
        ("es2021", &new_globals_2015_2017_2020_2021),
        ("es2022", &new_globals_2015_2017_2020_2021),
        ("es2023", &new_globals_2015_2017_2020_2021),
        ("es2024", &new_globals_2015_2017_2020_2021),
        // Platforms
        ("browser", &globals["browser"]),
        ("node", &globals["node"]),
        ("shared-node-browser", &globals["shared-node-browser"]),
        ("worker", &globals["worker"]),
        ("serviceworker", &globals["serviceworker"]),
        // Frameworks
        ("commonjs", &globals["commonjs"]),
        ("amd", &globals["amd"]),
        ("mocha", &globals["mocha"]),
        ("jasmine", &globals["jasmine"]),
        ("jest", &globals["jest"]),
        ("phantomjs", &globals["phantomjs"]),
        ("jquery", &globals["jquery"]),
        ("qunit", &globals["qunit"]),
        ("prototypejs", &globals["prototypejs"]),
        ("shelljs", &globals["shelljs"]),
        ("meteor", &globals["meteor"]),
        ("mongo", &globals["mongo"]),
        ("protractor", &globals["protractor"]),
        ("applescript", &globals["applescript"]),
        ("nashorn", &globals["nashorn"]),
        ("atomtest", &globals["atomtest"]),
        ("embertest", &globals["embertest"]),
        ("webextensions", &globals["webextensions"]),
        ("greasemonkey", &globals["greasemonkey"]),
    ]
    .iter()
    .map(|(name, vars)| Env { name, vars: to_env_vars(vars) })
    .collect();

    let context = Context::new(envs_preset);
    let template = template::Template::with_context(&context);
    if let Err(err) = template.render() {
        eprintln!("failed to render environments template: {err}");
    }
}

fn to_env_vars(env_var_map: &FxHashMap<String, bool>) -> Vec<EnvVar> {
    let mut result: Vec<EnvVar> = vec![];
    for (key, value) in env_var_map {
        result.push(EnvVar { name: key, writeable: *value });
    }

    result.sort_by(|a, b| a.name.cmp(b.name));
    result
}
