// Examples of incorrect code for switch-exhaustiveness-check rule

type Status = 'pending' | 'approved' | 'rejected';

function handleStatus(status: Status) {
  switch (status) {
    case 'pending':
      return 'Waiting for approval';
    case 'approved':
      return 'Request approved';
    // Missing 'rejected' case
  }
}