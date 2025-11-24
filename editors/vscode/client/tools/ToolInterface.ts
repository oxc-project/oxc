import { ConfigurationChangeEvent, ExtensionContext, LogOutputChannel } from 'vscode';
import { ConfigService } from '../ConfigService';
import StatusBarItemHandler from '../StatusBarItemHandler';

export default interface ToolInterface {
  /**
   * Activates the tool and initializes any necessary resources.
   */
  activate(
    context: ExtensionContext,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void>;

  /**
   * Deactivates the tool and cleans up any resources.
   */
  deactivate(): Promise<void>;

  /**
   * Toggles the tool's active state based on configuration.
   */
  toggleClient(configService: ConfigService): Promise<void>;

  /**
   * Restart the tool.
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
