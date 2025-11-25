import { LogOutputChannel, window } from "vscode";
import { Executable, MessageType, ShowMessageParams } from "vscode-languageclient/node";

export function runExecutable(path: string, nodePath?: string): Executable {
  const serverEnv: Record<string, string> = {
    ...process.env,
    RUST_LOG: process.env.RUST_LOG || "info",
  };
  if (nodePath) {
    serverEnv.PATH = `${nodePath}${process.platform === "win32" ? ";" : ":"}${process.env.PATH ?? ""}`;
  }
  const isNode = path.endsWith(".js") || path.endsWith(".cjs") || path.endsWith(".mjs");

  return isNode
    ? {
        command: "node",
        args: [path, "--lsp"],
        options: {
          env: serverEnv,
        },
      }
    : {
        command: path,
        args: ["--lsp"],
        options: {
          // On Windows we need to run the binary in a shell to be able to execute the shell npm bin script.
          // Searching for the right `.exe` file inside `node_modules/` is not reliable as it depends on
          // the package manager used (npm, yarn, pnpm, etc) and the package version.
          // The npm bin script is a shell script that points to the actual binary.
          // Security: We validated the userDefinedBinary in `configService.getUserServerBinPath()`.
          shell: process.platform === "win32",
          env: serverEnv,
        },
      };
}

export function onClientNotification(params: ShowMessageParams, outputChannel: LogOutputChannel) {
  switch (params.type) {
    case MessageType.Debug:
      outputChannel.debug(params.message);
      break;
    case MessageType.Log:
      outputChannel.info(params.message);
      break;
    case MessageType.Info:
      window.showInformationMessage(params.message);
      break;
    case MessageType.Warning:
      window.showWarningMessage(params.message);
      break;
    case MessageType.Error:
      window.showErrorMessage(params.message);
      break;
    default:
      outputChannel.info(params.message);
  }
}
