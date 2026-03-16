import { getAvenAccountData } from './dataReport/avenAccountDataReportManager';

export type InstallType = { x: number };

export const installmentLoanManager = { call: () => getAvenAccountData() };
