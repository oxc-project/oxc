//! Generator of envs for Oxlint JS plugins.

use convert_case::{Case, Casing};
use javascript_globals::GLOBALS;

use crate::{
    Codegen, Generator, OXLINT_APP_PATH,
    output::Output,
    schema::Schema,
    utils::{string, write_it},
};

use super::define_generator;

pub struct OxlintEnvsGenerator;

define_generator!(OxlintEnvsGenerator);

impl Generator for OxlintEnvsGenerator {
    fn generate(&self, _schema: &Schema, _codegen: &Codegen) -> Output {
        let code = generate();

        Output::Javascript { path: format!("{OXLINT_APP_PATH}/src-js/generated/envs.ts"), code }
    }
}

/// Generate environment definitions.
fn generate() -> String {
    #[rustfmt::skip]
    let mut out = string!("
        /**
         * Set of globals for an environment.
         */
        export interface EnvPreset {
            readonly: string[];
            writable: string[];
        }
    ");

    #[rustfmt::skip]
    let mut map = string!("
        /**
         * `Map` of variables defined by environments.
         */
        export const ENVS: Map<string, EnvPreset> = new Map([
    ");

    let mut envs = GLOBALS.entries().collect::<Vec<_>>();
    envs.sort_unstable_by_key(|(key, _)| **key);

    for (&name, vars) in envs {
        let const_name = format!("ENV_{}", name.to_case(Case::UpperSnake));

        let mut readonly_names = vec![];
        let mut writable_names = vec![];

        for (&var_name, &is_writable) in vars {
            if is_writable {
                writable_names.push(var_name);
            } else {
                readonly_names.push(var_name);
            }
        }

        #[rustfmt::skip]
        write_it!(out, "
            /**
             * `{name}` environment.
             */
            export const {const_name}: EnvPreset = {{
                readonly: [
        ");

        if !readonly_names.is_empty() {
            readonly_names.sort_unstable();
            write_it!(out, "\"{}\",\n", readonly_names.join("\",\n\""));
        }

        out.push_str("],\nwritable: [\n");

        if !writable_names.is_empty() {
            writable_names.sort_unstable();
            write_it!(out, "\"{}\",\n", writable_names.join("\",\n\""));
        }

        out.push_str("],\n};\n\n");

        write_it!(map, "[\"{name}\", {const_name}],\n");
    }

    map.push_str("]);\n");
    out.push_str(&map);

    out
}
