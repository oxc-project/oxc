(
  options,
  { log, logger, messenger }: {
    log: LogFun;
    logger: Logger;
    messenger: Messenger;
  }) => {

}

export function useCopyToClipboard({ timeout = 2000, onCopy }: {
  timeout?: number;
  onCopy?: () => void;
} = {}) {}
