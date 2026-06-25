import { installmentLoanManager } from './installmentLoanManager';
import { aaaInternal } from './aaaInternal';

export const balanceSweepDetailsManager = {
  call(): string {
    return installmentLoanManager.call() + aaaInternal.call();
  },
};
