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
} = {}) { }

function callbackUrl(
  { baseUrl, params }: { baseUrl: string; params?: string } = {
    baseUrl: "",
    params: undefined,
  }
) { }

// Single hugged pattern: a source-inline type literal stays flat (contrast: useCopyToClipboard above)
export default function useTagsCount({
  query,
}: { query?: Record<any, any> } = {}) {
}

function parseTitle(
  item: PageObjectResponse | DatabaseObjectResponse,
  {
    maxLength = DocumentValidation.maxTitleLength,
  }: { maxLength?: number } = {}
) {}
