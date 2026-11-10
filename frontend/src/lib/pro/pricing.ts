// CAMark Pro pricing as shown in the app — v4.1, mirrors
// `0 - Bussines Plan/08-pricing/01-master-pricing-strategy.md` (the single source of truth).
// CAMark is Tier 3: its License Key (1 device) prices are lower than CATerm / CAShell, while the
// 1-month License and the Subscription (5 devices) are the same for all three products.
// These numbers are display only: the amounts actually charged live in the checkout gateways
// (Mayar / QRIS for IDR, Creem / Ko-fi for USD) and must be changed together with them, never
// on their own.
//
// AI is 100% BYOK (Bring Your Own Key): no plan includes an AI request quota.

export type Currency = 'USD' | 'IDR';
/** License Key terms (1 device, offline HWID activation). */
export type LicenseTerm = 'lifetime' | 'yearly' | 'monthly';
/** Subscription billing intervals (5 devices, GCC cloud account). */
export type BillingInterval = 'monthly' | 'yearly';

export interface PricePoint {
  readonly normal: number;
  readonly promo: number;
}

export type DualPrice = Readonly<Record<Currency, PricePoint>>;

export const PRO_PRICING = {
  /**
   * Whether the promo (struck-through) prices are shown. Keep `false` unless the gateways are
   * charging the promo amounts right now (BFCM / launch campaign).
   */
  promoActive: false,
  license: {
    devices: 1,
    lifetime: { USD: { normal: 7.9, promo: 2.9 }, IDR: { normal: 119_000, promo: 45_000 } },
    yearly: { USD: { normal: 4.9, promo: 1.9 }, IDR: { normal: 79_000, promo: 35_000 } },
    monthly: { USD: { normal: 1.9, promo: 0.99 }, IDR: { normal: 29_000, promo: 15_000 } }
  },
  subscription: {
    devices: 5,
    monthly: { USD: { normal: 10, promo: 4.9 }, IDR: { normal: 149_000, promo: 69_000 } },
    yearly: { USD: { normal: 120, promo: 79 }, IDR: { normal: 1_790_000, promo: 1_190_000 } },
    /** The monthly promo lasts this many months; from month 13 it returns to the normal price. */
    promoMonths: 12
  },
  /** Checkout gateway per currency. */
  gateway: { IDR: 'Mayar (QRIS)', USD: 'Creem / Ko-fi' },
  trialDays: 7,
  /** One account covers up to this many people (owner + 6), at no extra cost. */
  maxMembers: 7,
  /** Devices a Pro subscription syncs across. */
  syncDevices: 5
} as const;

export const LICENSE_TERMS: readonly LicenseTerm[] = ['lifetime', 'yearly', 'monthly'];

/** `$10`, `$7.9`, `$0.99` — whole dollars without decimals, otherwise at most two decimals. */
export function formatUsd(amount: number): string {
  return `$${Number(amount.toFixed(2))}`;
}

/** `Rp 119.000`, `Rp 1.790.000` — whole rupiah with `.` thousands separators. */
export function formatIdr(amount: number): string {
  const digits = Math.round(amount)
    .toString()
    .replace(/\B(?=(\d{3})+(?!\d))/g, '.');
  return `Rp ${digits}`;
}

export function formatPrice(amount: number, currency: Currency): string {
  return currency === 'IDR' ? formatIdr(amount) : formatUsd(amount);
}

/** IDR for the Indonesian UI, USD otherwise. */
export function defaultCurrency(locale: string): Currency {
  return locale === 'id' ? 'IDR' : 'USD';
}

/**
 * The price to display for one entry: `current` is what the customer pays now, `original` is the
 * struck-through normal price while a promo is active (otherwise `null`).
 */
export function displayPrice(
  price: DualPrice,
  currency: Currency,
  promoActive: boolean = PRO_PRICING.promoActive
): { current: string; original: string | null } {
  const { normal, promo } = price[currency];
  return promoActive
    ? { current: formatPrice(promo, currency), original: formatPrice(normal, currency) }
    : { current: formatPrice(normal, currency), original: null };
}
