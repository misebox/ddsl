import { workspace, type ExtensionContext } from "vscode";
import {
  LanguageClient,
  TransportKind,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(_context: ExtensionContext): void {
  const command = workspace
    .getConfiguration("nounsql")
    .get<string>("server.path", "nounsql-lsp");

  const serverOptions: ServerOptions = {
    run: { command, transport: TransportKind.stdio },
    debug: { command, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "nounsql" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.nsql"),
    },
  };

  client = new LanguageClient("nounsql", "NounSQL", serverOptions, clientOptions);
  void client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
