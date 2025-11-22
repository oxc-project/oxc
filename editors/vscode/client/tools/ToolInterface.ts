import { ConfigurationChangeEvent, ExtensionContext, LogOutputChannel } from 'vscode';
import { ConfigService } from '../ConfigService';
import StatusBarItemHandler from '../StatusBarItemHandler';

export default interface ToolInterface {
  /**
   * Activates the tool and creates an LSP connection if necessary.
   */
  activate(
    context: ExtensionContext,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void>;

  /**
   * Deactivates the tool and stops the LSP connection if necessary.
   */
  deactivate(): Promise<void>;

  /**
   * Starts or stops the LSP client.
   */
  toggleClient(configService: ConfigService): Promise<void>;

  /**
   * Restart the LSP client.
   */
  restartClient(): Promise<void>;

  /**
   * Handles configuration changes.
   */
  onConfigChange(
    event: ConfigurationChangeEvent,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void>;
}
