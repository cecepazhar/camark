// Drives the boot splash defined inline in app.html. The splash stays up while the app warms
// up (at least MIN_MS so it never flickers, never more than MAX_MS — app.html enforces the cap
// on its own too, so a stuck promise can never keep the user waiting).

const MIN_MS = 1000;
const MAX_MS = 2800;

type BootApi = { start: number; lang?: string; step: (text: string, pct?: number) => void; done: () => void };

// The splash runs before i18n is loaded, so its few lines are kept here.
const STEPS = {
  en: { workspace: 'Loading workspace…', editor: 'Preparing editor…', fonts: 'Loading fonts…', ready: 'Ready' },
  id: { workspace: 'Memuat workspace…', editor: 'Menyiapkan editor…', fonts: 'Memuat font…', ready: 'Siap' }
};
const api = (): BootApi | undefined => (typeof window !== 'undefined' ? (window as any).__camarkBoot : undefined);

// rAF never fires while the window is hidden/occluded, so cap the wait.
const frame = () =>
  Promise.race([
    new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))),
    new Promise<void>((r) => setTimeout(r, 120))
  ]);
const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
const settle = (p: Promise<unknown>) => p.then(() => undefined, () => undefined);

/** Warm-up work that makes the first interactions instant; each step is best-effort. */
async function warmUp(step: BootApi['step'], lang?: string) {
  const s = lang === 'id' ? STEPS.id : STEPS.en;
  step(s.workspace, 35);
  // Restore open document tabs now so the editor opens with them already parsed.
  await settle(import('$lib/stores/docTabs.svelte'));

  step(s.editor, 60);
  await settle(
    Promise.all([
      import('@codemirror/view'),
      import('@codemirror/state'),
      import('@codemirror/commands'),
      import('@codemirror/language'),
      import('@codemirror/lang-markdown'),
      // KaTeX is lazy (see markdownRender.ts); fetch it now so the first preview is instant.
      import('$lib/utils/markdownRender').then((m) => m.ensureKatex())
    ])
  );

  step(s.fonts, 80);
  if (document.fonts?.ready) await Promise.race([settle(document.fonts.ready), sleep(400)]);

  step(s.ready, 95);
  await frame(); // let the real UI paint underneath before the splash fades
}

let finished = false;

export async function finishBoot(): Promise<void> {
  const boot = api();
  if (!boot || finished) return;
  finished = true;
  await Promise.race([warmUp(boot.step, boot.lang), sleep(MAX_MS)]);
  const elapsed = performance.now() - boot.start;
  if (elapsed < MIN_MS) await sleep(MIN_MS - elapsed);
  boot.done();
}
