// Utility functions for calculating pay period numbers and total pay periods

export interface PayPeriodInfo {
  payPeriodNumber: number;
  totalPayPeriods: number;
  payFrequency: string;
}

export function calculatePayPeriodInfo(
  startDate: string,
  endDate: string
): PayPeriodInfo {
  const start = new Date(startDate + 'T00:00:00');
  const end = new Date(endDate + 'T00:00:00');

  if (isNaN(start.getTime()) || isNaN(end.getTime())) {
    throw new Error('Invalid date format');
  }

  if (start > end) {
    throw new Error('Start date must be before end date');
  }

  const days = (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24);
  
  let payFrequency: string;
  if (days < 8) {
    payFrequency = 'Weekly';
  } else if (days < 15) {
    payFrequency = 'BiWeekly';
  } else if (days < 16) {
    payFrequency = 'SemiMonthly';
  } else {
    payFrequency = 'Monthly';
  }

  const yearStart = new Date(start.getFullYear(), 0, 1);
  let payPeriodNumber: number;
  let totalPayPeriods: number;

  switch (payFrequency) {
    case 'Weekly':
      payPeriodNumber = getWeekNumber(start);
      totalPayPeriods = isLeapYear(start.getFullYear()) ? 53 : 52;
      break;
    case 'BiWeekly':
      const weeksFromYearStart = Math.floor((start.getTime() - yearStart.getTime()) / (7 * 24 * 60 * 60 * 1000));
      payPeriodNumber = Math.floor(weeksFromYearStart / 2) + 1;
      totalPayPeriods = 26;
      break;
    case 'SemiMonthly':
      const monthDay = start.getDate();
      payPeriodNumber = (start.getMonth() * 2) + (monthDay <= 15 ? 1 : 2);
      totalPayPeriods = 24;
      break;
    case 'Monthly':
      payPeriodNumber = start.getMonth() + 1;
      totalPayPeriods = 12;
      break;
    default:
      payPeriodNumber = 1;
      totalPayPeriods = 12;
  }

  return { payPeriodNumber, totalPayPeriods, payFrequency };
}

function getWeekNumber(date: Date): number {
  const d = new Date(date);
  d.setHours(0, 0, 0, 0);
  d.setDate(d.getDate() + 3 - (d.getDay() + 6) % 7);
  const week1 = new Date(d.getFullYear(), 0, 4);
  return Math.round(((d.getTime() - week1.getTime()) / 86400000 - 3 + (week1.getDay() + 6) % 7) / 7) + 1;
}

function isLeapYear(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || (year % 400 === 0);
}

export function formatPayPeriod(periodNumber: number, totalPeriods: number, frequency: string): string {
  return `Period ${periodNumber} of ${totalPeriods} (${frequency})`;
}
