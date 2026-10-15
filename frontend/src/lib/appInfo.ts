// Single source of truth for version string prints. Was hardcoded in four places
// and drifted (sidebar said v2.1.7 while build was 2.1.8). package.json bumped in
// lockstep with Cargo workspace / tauri.conf.json on release.
import { version } from '../../package.json';

export const APP_VERSION: string = version;
export const REPO_URL = 'https://github.com/cecepazhar/camark';
export const WEBSITE_URL = 'https://camark.fathforce.com';
export const AUTHOR_NAME = 'Cecep Saeful Azhar Hidayat, ST';
export const AUTHOR_URL = 'https://www.cecepazhar.com';
export const CONTACT_EMAIL = 'hi@cecepazhar.com';
export const CONTACT_PHONE = '0852 2069 6117';
export const INCUBATOR_NAME = 'Fathforce';
export const INCUBATOR_URL = 'https://fathforce.com';
export const PRICING_URL = 'https://www.cecepazhar.com/#pricing';
export const GITHUB_SPONSORS_URL = '';
export const KOFI_URL = '';
export const PAYPAL_URL = '';

/**
 * Pricing page for the signed-in CAMark account: the landing page passes the account id on to
 * Lemon Squeezy as checkout custom data, so the subscription lands on this account whatever email
 * is typed at checkout. Only the opaque account id travels, never the email.
 */
export function pricingUrl(accountId?: string | null): string {
  if (!accountId || !/^[0-9a-f-]{36}$/i.test(accountId)) return PRICING_URL;
  return `${WEBSITE_URL}/?from=app&account=${accountId}#pricing`;
}

export function releaseNotesUrl(ver: string): string {
  return `${REPO_URL}/releases/tag/v${ver.replace(/^v/, '')}`;
}
