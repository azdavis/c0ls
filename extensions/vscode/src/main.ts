import { ExtensionContext, workspace } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { join } from "path";

// TODO support many roots
let client: LanguageClient | null = null;

export function activate(cx: ExtensionContext) {
  if (client !== null) {
    return;
  }
  const serverOpts: ServerOptions = {
    command: cx.asAbsolutePath(join("out", "c0ls")),
  };
  const clientOpts: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "c0" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.{c0,h0}"),
    },
  };
  client = new LanguageClient("c0ls", serverOpts, clientOpts, true);
  cx.subscriptions.push(client.start());
}

export function deactivate(): Promise<void> {
  if (client === null) {
    return Promise.resolve();
  }
  return client.stop();
}
