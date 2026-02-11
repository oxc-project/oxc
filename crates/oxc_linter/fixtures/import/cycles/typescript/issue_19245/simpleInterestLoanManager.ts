import type { InstallType } from './installmentLoanManager';

export const simpleInterestLoanManager = {
  call: (): string => String(Boolean(null as InstallType | null)),
};
