import { consola } from "consola";
import { cp, mkdir, rm, writeFile } from 'fs/promises';
import { join, resolve } from 'path';
import { spawn } from 'promisify-child-process';
import packageJson from "../package.json";

type BuildTarget = "nodejs" | "bundler"

const outDir = resolve(join(__dirname, "../../../npm/wasm-parser"))

async function main() {
    consola.start("Building WASM target ...");

    await rm(outDir, { recursive: true, force: true })
    await mkdir(outDir, { recursive: true })

    await Promise.all([
        buildProject("bundler", join(outDir, "bundler")),
        buildProject("nodejs", join(outDir, "node"))
    ])

    consola.info("Moving package.json ...")

    await moveStaticFiles(outDir)

    consola.success("Built the project.")
}

async function buildProject(target: BuildTarget, outDir: string) {
    await spawn("wasm-pack", ["build", "--no-pack", "--target", target, "--out-dir", outDir], { stdio: "ignore" })
    await rm(join(outDir, ".gitignore"))
}

async function moveStaticFiles(outDir: string) {
    const pacakgeJsonPath = join(outDir, "package.json")
    await writeFile(pacakgeJsonPath, JSON.stringify(packageJson, null, 2))

    const readmeFilePath = join(outDir, "README.md")
    await cp("README.md", readmeFilePath)

}

main()