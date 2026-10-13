<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import ProfileAvatar from '$lib/components/ProfileAvatar.svelte';
  import AvatarPicker from '$lib/components/AvatarPicker.svelte';
  import AmbientSettingsCard from '$lib/components/AmbientSettingsCard.svelte';
  import { getProfile, saveProfile } from '$lib/stores/profile.svelte';
  import { getPro } from '$lib/stores/pro.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { changeMasterPassword, MIN_VAULT_PASSWORD_LEN } from '$lib/api/vault';
  import { getAiSettings, saveAiSettings, testAiConnection, type AiSettings } from '$lib/api/ai';
  import { listProfiles, saveProfile as saveFamilyProfile, type ProfileRecord } from '$lib/api/profiles';
  import { exportEncryptedBackup, importEncryptedBackup } from '$lib/api/prefs';
  import { proActivateLicense, proLogout, type ProAccount } from '$lib/api/pro';
  import ProLoginForm from '$lib/components/ProLoginForm.svelte';
  import {
    APP_VERSION,
    REPO_URL,
    WEBSITE_URL,
    AUTHOR_NAME,
    AUTHOR_URL,
    CONTACT_EMAIL,
    CONTACT_PHONE,
    INCUBATOR_NAME,
    INCUBATOR_URL
  } from '$lib/appInfo';
  import { relaunch } from '@tauri-apps/plugin-process';

  // Surface tokens aligning 100% with CATerm architecture and design system
  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-xs dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const LABEL_BASE = 'block text-xs font-semibold uppercase tracking-wider text-neutral-600 dark:text-neutral-400';
  const LABEL = `${LABEL_BASE} mb-1.5`;
  const INPUT = 'w-full px-3.5 py-2.5 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-sm text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-cyan-500 transition-colors';
  const EYE_BUTTON = 'absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors p-1.5 cursor-pointer';

  const TABS = ['profile', 'pro', 'security', 'appearance', 'profiles', 'ai', 'backup', 'about'] as const;
  type SettingsTab = (typeof TABS)[number];

  const requestedTab = page.url.searchParams.get('tab') ?? '';
  let activeTab = $state<SettingsTab>(
    (TABS as readonly string[]).includes(requestedTab) ? (requestedTab as SettingsTab) : 'profile'
  );

  function switchTab(tab: SettingsTab) {
    activeTab = tab;
    const url = new URL(window.location.href);
    url.searchParams.set('tab', tab);
    void goto(url.toString(), { replaceState: true, noScroll: true, keepFocus: true });
  }

  // License & Pro State
  let licenseKeyInput = $state('');
  let isActivatingLicense = $state(false);

  async function handleActivateLicense() {
    if (!licenseKeyInput.trim()) return;
    isActivatingLicense = true;
    try {
      const status = await proActivateLicense(licenseKeyInput.trim());
      showToast('Lisensi Pro berhasil diaktivasi!', 'success');
      licenseKeyInput = '';
      window.location.reload();
    } catch (e: any) {
      alert(`Gagal aktivasi lisensi: ${e?.message || e}`);
    } finally {
      isActivatingLicense = false;
    }
  }

  async function handleProSignedIn(account: ProAccount) {
    showToast(`Berhasil masuk sebagai ${account.name}!`, 'success');
    window.location.reload();
  }

  async function handleProLogout() {
    try {
      await proLogout();
      showToast('Berhasil keluar akun Pro', 'info');
      window.location.reload();
    } catch (e: any) {
      alert(`Gagal keluar: ${e}`);
    }
  }

  // Profile State
  const profile = getProfile();
  const pro = getPro();
  let profileName = $state(profile.name);
  let profileAvatar = $state(profile.avatar);
  const profileDirty = $derived(profileName.trim() !== profile.name || profileAvatar !== profile.avatar);

  function handleSaveProfile(e: Event) {
    e.preventDefault();
    if (!profileName.trim()) return;
    saveProfile({ name: profileName, avatar: profileAvatar });
    profileName = profile.name;
    showToast(t('settings.profile.saved'), 'success');
  }

  // Master Password State
  let oldPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let showOldPassword = $state(false);
  let showNewPassword = $state(false);
  let showConfirmPassword = $state(false);
  let isChangingPassword = $state(false);

  async function handleChangeMasterPassword(e: Event) {
    e.preventDefault();
    if (newPassword.length < MIN_VAULT_PASSWORD_LEN) {
      showToast(t('settings.security.errMin', { min: MIN_VAULT_PASSWORD_LEN }), 'error');
      return;
    }
    if (newPassword !== confirmPassword) {
      showToast(t('settings.security.errMismatch'), 'error');
      return;
    }

    const confirmed = await confirmModal(
      t('settings.security.rekeyNote'),
      t('settings.security.title'),
      false,
      t('settings.security.updateButton'),
      t('common.cancel')
    );
    if (!confirmed) return;

    isChangingPassword = true;
    try {
      await changeMasterPassword(oldPassword, newPassword);
      oldPassword = '';
      newPassword = '';
      confirmPassword = '';

      const restartNow = await confirmModal(
        t('settings.security.changedConfirm'),
        t('settings.security.changedTitle'),
        false,
        t('settings.security.restartNow'),
        t('settings.security.later')
      );
      if (restartNow) {
        try {
          await relaunch();
        } catch {
          window.location.reload();
        }
        return;
      }
      showToast(t('settings.security.changedToast'), 'success');
    } catch (err: any) {
      showToast(err?.message || 'Failed to update master password', 'error');
    } finally {
      isChangingPassword = false;
    }
  }

  // Family Profiles State
  let profilesList = $state<ProfileRecord[]>([]);
  let isAddProfileOpen = $state(false);
  let newProfileName = $state('');
  let newProfileRole = $state<'owner' | 'partner' | 'member' | 'child'>('member');
  let newProfilePin = $state('');

  async function handleAddProfile() {
    if (!newProfileName.trim()) {
      showToast(t('settings.profilesTab.nameRequired'), 'error');
      return;
    }
    try {
      await saveFamilyProfile(
        {
          name: newProfileName.trim(),
          role: newProfileRole,
          pin: newProfilePin.trim() || undefined,
          avatar: ''
        },
        '00000000-0000-0000-0000-000000000000'
      );
      showToast(t('settings.profilesTab.addedToast'), 'success');
      newProfileName = '';
      newProfilePin = '';
      isAddProfileOpen = false;
      profilesList = await listProfiles();
    } catch (e: any) {
      showToast(e?.message || 'Failed to add profile', 'error');
    }
  }

  // AI Configuration State
  let aiSettings = $state<AiSettings>({
    mode: 'byo',
    base_url: 'http://localhost:20128/v1',
    model: 'ZA126_PRO',
    has_api_key: false
  });
  let aiApiKeyInput = $state('');
  let showAiApiKey = $state(false);
  let isSavingAi = $state(false);
  let isTestingAi = $state(false);
  let aiTestResult = $state<{ ok: boolean; message: string } | null>(null);

  async function handleSaveAi(e: Event) {
    e.preventDefault();
    isSavingAi = true;
    try {
      // Force mode to byo when saving custom config
      aiSettings.mode = 'byo';
      await saveAiSettings(aiSettings, aiApiKeyInput);
      aiApiKeyInput = '';
      aiSettings = await getAiSettings();
      showToast(t('settings.ai.savedToast'), 'success');
    } catch (e: any) {
      showToast(e?.message || 'Failed to save AI settings', 'error');
    } finally {
      isSavingAi = false;
    }
  }

  async function handleTestAi() {
    isTestingAi = true;
    aiTestResult = null;
    try {
      const keyToSend = aiApiKeyInput.trim().length > 0 ? aiApiKeyInput : undefined;
      const msg = await testAiConnection(aiSettings.base_url, aiSettings.model, keyToSend);
      aiTestResult = { ok: true, message: msg };
      showToast(t('settings.ai.testOk'), 'success');
    } catch (e: any) {
      aiTestResult = { ok: false, message: e?.message || 'Connection failed' };
      showToast(t('settings.ai.testFail'), 'error');
    } finally {
      isTestingAi = false;
    }
  }

  // Backup & Restore State
  let exportPassphrase = $state('');
  let showExportPassphrase = $state(false);
  let isExporting = $state(false);

  let restorePassphrase = $state('');
  let showRestorePassphrase = $state(false);
  let isRestoring = $state(false);
  let selectedBackupFile = $state<File | null>(null);

  async function handleExportBackup(e: Event) {
    e.preventDefault();
    if (exportPassphrase.length < 8) {
      showToast(t('settings.backup.errPassphraseMin'), 'error');
      return;
    }
    isExporting = true;
    try {
      showToast(t('settings.backup.exporting'), 'info');
      const dataBytes = await exportEncryptedBackup(exportPassphrase);
      const uint8Array = new Uint8Array(dataBytes);
      const blob = new Blob([uint8Array], { type: 'application/octet-stream' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `camark-backup-${new Date().toISOString().slice(0, 10)}.cafbackup`;
      a.click();
      URL.revokeObjectURL(url);
      exportPassphrase = '';
      showToast(t('settings.backup.exportSuccess'), 'success');
    } catch (e: any) {
      showToast(e?.message || 'Failed to export backup', 'error');
    } finally {
      isExporting = false;
    }
  }

  function handleFileSelected(event: Event) {
    const target = event.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      selectedBackupFile = target.files[0];
    }
  }

  async function handleImportBackup(e: Event) {
    e.preventDefault();
    if (!selectedBackupFile) {
      showToast('Please select a .cafbackup file first', 'error');
      return;
    }
    if (restorePassphrase.length < 8) {
      showToast(t('settings.backup.errPassphraseMin'), 'error');
      return;
    }

    isRestoring = true;
    try {
      showToast(t('settings.backup.restoring'), 'info');
      const arrayBuffer = await selectedBackupFile.arrayBuffer();
      const bytes = Array.from(new Uint8Array(arrayBuffer));
      await importEncryptedBackup(bytes, restorePassphrase);
      restorePassphrase = '';
      selectedBackupFile = null;
      showToast(t('settings.backup.restoreSuccess'), 'success');
      profilesList = await listProfiles().catch(() => []);
    } catch (e: any) {
      showToast(e?.message || 'Failed to restore backup', 'error');
    } finally {
      isRestoring = false;
    }
  }

  onMount(async () => {
    try {
      const [ai, profs] = await Promise.all([
        getAiSettings().catch(() => aiSettings),
        listProfiles().catch(() => [])
      ]);
      aiSettings = ai;
      profilesList = profs;
    } catch {}
  });
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <!-- Page Header using Standardized CATerm Tile Design -->
  <PageHeader
    icon={[
      'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z',
      'M15 12a3 3 0 11-6 0 3 3 0 016 0z'
    ]}
    accent="cyan"
    title={t('settings.title')}
    subtitle={t('settings.subtitle')}
  />

  <!-- Horizontal Scrollable Navigation Tabs -->
  <div class="border-b border-neutral-200 dark:border-neutral-800 flex gap-4 overflow-x-auto scrollbar-none" role="tablist">
    {#each TABS as tab (tab)}
      <button
        type="button"
        role="tab"
        aria-selected={activeTab === tab}
        onclick={() => switchTab(tab)}
        class="pb-3 whitespace-nowrap text-sm font-medium transition-colors border-b-2 {activeTab === tab
          ? 'border-cyan-500 text-neutral-900 dark:text-white font-semibold'
          : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
      >
        {tab === 'pro' ? 'Pro & Lisensi' : t(`settings.tabs.${tab}`)}
      </button>
    {/each}
  </div>

  <!-- Settings Tab Content Panes -->
  {#if activeTab === 'profile'}
    <div class="{CARD} space-y-6">
      <div class="flex items-center gap-4">
        <ProfileAvatar avatar={profileAvatar} name={profileName} size={56} pro={pro.isPro || profile.plan === 'pro'} />
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-neutral-900 dark:text-white truncate">
              {profileName.trim() || profile.name}
            </h2>
            {#if pro.isPro || profile.plan === 'pro'}
              <span class="text-[10px] font-bold tracking-wider px-2 py-0.5 rounded-full border border-amber-400/50 bg-amber-400/10 text-amber-500 uppercase shadow-xs">
                PRO
              </span>
            {:else}
              <span class="text-[10px] font-semibold tracking-wider px-2 py-0.5 rounded-full border border-neutral-300 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 uppercase">
                {t('profileMenu.planFree')}
              </span>
            {/if}
          </div>
          <p class="{MUTED} text-sm mt-0.5">{t('settings.profile.localNote')}</p>
        </div>
      </div>

      <form onsubmit={handleSaveProfile} class="space-y-5 max-w-md">
        <div>
          <label for="profile-name-input" class={LABEL}>{t('settings.profile.displayName')}</label>
          <input
            id="profile-name-input"
            type="text"
            maxlength="48"
            bind:value={profileName}
            placeholder="CAMark User"
            class={INPUT}
          />
        </div>

        <div>
          <span class="{LABEL_BASE} mb-2">{t('settings.profile.picture')}</span>
          <AvatarPicker bind:value={profileAvatar} size={40} />
        </div>

        <button
          type="submit"
          disabled={!profileDirty}
          class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 disabled:hover:bg-cyan-600 text-white text-sm font-medium rounded-lg transition-colors shadow-xs cursor-pointer"
        >
          {t('settings.profile.save')}
        </button>
      </form>

      <div class="pt-5 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between gap-4">
        <div>
          <p class="text-sm font-medium text-neutral-900 dark:text-white">{t('settings.profile.masterPassword')}</p>
          <p class="text-xs {MUTED}">{t('settings.profile.masterPasswordBody')}</p>
        </div>
        <button
          type="button"
          onclick={() => switchTab('security')}
          class="px-3.5 py-1.5 text-xs font-medium rounded-lg border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
        >
          {t('settings.profile.changeMasterPassword')}
        </button>
      </div>
    </div>
  {:else if activeTab === 'pro'}
    <div class="{CARD} space-y-6">
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-4">
        <div>
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">CAMark Pro & Lisensi GCC Cloud</h2>
            {#if pro.isPro || profile.plan === 'pro'}
              <span class="text-xs font-bold px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-500 border border-emerald-500/30 uppercase">
                Aktif ✓
              </span>
            {:else}
              <span class="text-xs font-semibold px-2 py-0.5 rounded-full bg-neutral-200 dark:bg-neutral-800 text-neutral-500 uppercase">
                Free Edition
              </span>
            {/if}
          </div>
          <p class="{MUTED} text-sm mt-1">
            Fitur Pro mencakup Native PDF Print, AI Contextual Copilot, Table Tools, dan Sync Brankas SQLCipher.
          </p>
        </div>
      </div>

      <!-- License Key Activation Box -->
      <div class="{SUBCARD} space-y-3">
        <h3 class="text-sm font-bold text-neutral-900 dark:text-white flex items-center gap-2">
          <span>🔑</span>
          <span>Aktivasi License Key</span>
        </h3>
        <p class="text-xs {MUTED}">
          Masukkan kode lisensi resmi dari Ko-fi, Mayar, atau Admin untuk membuka seluruh fitur Pro.
        </p>

        <div class="flex items-center gap-2 pt-1 max-w-lg">
          <input
            type="text"
            bind:value={licenseKeyInput}
            placeholder="CMRK-PRO-XXXX-XXXX-XXXX"
            class="{INPUT} font-mono uppercase"
          />
          <button
            type="button"
            disabled={isActivatingLicense || !licenseKeyInput.trim()}
            onclick={handleActivateLicense}
            class="px-4 py-2.5 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white text-xs font-bold rounded-lg transition-colors shrink-0 cursor-pointer"
          >
            {isActivatingLicense ? 'Aktivasi...' : 'Aktivasi'}
          </button>
        </div>
      </div>

      <!-- GCC Cloud Account Section -->
      <div class="{SUBCARD} space-y-4">
        <h3 class="text-sm font-bold text-neutral-900 dark:text-white flex items-center gap-2">
          <span>☁️</span>
          <span>Akun GCC Cloud Hub</span>
        </h3>

        {#if pro.status?.signedIn && pro.status?.account}
          <div class="flex items-center justify-between p-3 rounded-lg bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
            <div>
              <p class="text-xs font-bold text-neutral-900 dark:text-white">{pro.status.account.name}</p>
              <p class="text-xs {MUTED}">{pro.status.account.email}</p>
            </div>
            <button
              type="button"
              onclick={handleProLogout}
              class="px-3 py-1.5 rounded-lg border border-red-500/30 text-red-500 hover:bg-red-500/10 text-xs font-medium cursor-pointer"
            >
              Keluar Akun
            </button>
          </div>
        {:else}
          <ProLoginForm onSignedIn={handleProSignedIn} />
        {/if}
      </div>
    </div>
  {:else if activeTab === 'security'}
    <div class="{CARD} space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.security.title')}</h2>
        <p class="{MUTED} text-sm mt-1">{t('settings.security.subtitle')}</p>
      </div>

      <div class="{SUBCARD} space-y-4">
        <p class="text-xs {MUTED} leading-relaxed">{t('settings.security.rekeyNote')}</p>

        <form onsubmit={handleChangeMasterPassword} class="max-w-md space-y-4 pt-1">
          <div>
            <label for="old-pass" class={LABEL}>{t('settings.security.current')}</label>
            <div class="relative">
              <input
                id="old-pass"
                type={showOldPassword ? 'text' : 'password'}
                required
                autocomplete="current-password"
                bind:value={oldPassword}
                placeholder="••••••••••••"
                class="{INPUT} pr-10"
              />
              <button
                type="button"
                onclick={() => (showOldPassword = !showOldPassword)}
                class={EYE_BUTTON}
                aria-label={showOldPassword ? 'Hide current password' : 'Show current password'}
              >
                {#if showOldPassword}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                {/if}
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label for="new-pass" class={LABEL}>{t('settings.security.new', { min: MIN_VAULT_PASSWORD_LEN })}</label>
              <div class="relative">
                <input
                  id="new-pass"
                  type={showNewPassword ? 'text' : 'password'}
                  required
                  minlength={MIN_VAULT_PASSWORD_LEN}
                  autocomplete="new-password"
                  bind:value={newPassword}
                  placeholder="••••••••••••"
                  class="{INPUT} pr-10"
                />
                <button
                  type="button"
                  onclick={() => (showNewPassword = !showNewPassword)}
                  class={EYE_BUTTON}
                  aria-label={showNewPassword ? 'Hide new password' : 'Show new password'}
                >
                  {#if showNewPassword}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  {/if}
                </button>
              </div>
            </div>

            <div>
              <label for="confirm-pass" class={LABEL}>{t('settings.security.confirm')}</label>
              <div class="relative">
                <input
                  id="confirm-pass"
                  type={showConfirmPassword ? 'text' : 'password'}
                  required
                  minlength={MIN_VAULT_PASSWORD_LEN}
                  autocomplete="new-password"
                  bind:value={confirmPassword}
                  placeholder="••••••••••••"
                  class="{INPUT} pr-10"
                />
                <button
                  type="button"
                  onclick={() => (showConfirmPassword = !showConfirmPassword)}
                  class={EYE_BUTTON}
                  aria-label={showConfirmPassword ? 'Hide confirm password' : 'Show confirm password'}
                >
                  {#if showConfirmPassword}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  {/if}
                </button>
              </div>
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button
              type="submit"
              disabled={isChangingPassword || !newPassword || !oldPassword}
              class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-colors shadow-xs cursor-pointer"
            >
              {isChangingPassword ? t('settings.security.updating') : t('settings.security.updateButton')}
            </button>
          </div>
        </form>
      </div>
    </div>
  {:else if activeTab === 'appearance'}
    <div class="space-y-6">
      <AmbientSettingsCard defaultAccent="#06b6d4" appName="CAMark" />
    </div>
  {:else if activeTab === 'profiles'}
    <div class="{CARD} space-y-6">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.profilesTab.title')}</h2>
          <p class="{MUTED} text-sm mt-0.5">{t('settings.profilesTab.subtitle')}</p>
        </div>
        <button
          type="button"
          onclick={() => (isAddProfileOpen = !isAddProfileOpen)}
          class="px-3.5 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold rounded-lg transition-colors shadow-xs shrink-0 cursor-pointer"
        >
          {isAddProfileOpen ? t('common.cancel') : `+ ${t('settings.profilesTab.addMember')}`}
        </button>
      </div>

      {#if isAddProfileOpen}
        <div class="{SUBCARD} space-y-4 animate-in fade-in duration-150">
          <h3 class="{LABEL_BASE} text-cyan-600 dark:text-cyan-400">{t('settings.profilesTab.newProfileTitle')}</h3>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <input
              type="text"
              bind:value={newProfileName}
              placeholder={t('settings.profilesTab.namePlaceholder')}
              class={INPUT}
            />
            <select
              bind:value={newProfileRole}
              class={INPUT}
            >
              <option value="owner">{t('profiles.roles.owner')} (Ayah)</option>
              <option value="partner">{t('profiles.roles.partner')} (Ibu)</option>
              <option value="member">{t('profiles.roles.member')}</option>
              <option value="child">{t('profiles.roles.child')} (Anak)</option>
            </select>
          </div>
          <div>
            <input
              type="password"
              bind:value={newProfilePin}
              maxlength="6"
              placeholder={t('settings.profilesTab.pinPlaceholder')}
              class="{INPUT} font-mono"
            />
          </div>
          <div class="flex justify-end gap-2.5 pt-1">
            <button
              type="button"
              onclick={() => (isAddProfileOpen = false)}
              class="px-3.5 py-1.5 bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 text-xs font-medium rounded-lg cursor-pointer"
            >
              {t('common.cancel')}
            </button>
            <button
              type="button"
              onclick={handleAddProfile}
              class="px-3.5 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold rounded-lg cursor-pointer"
            >
              {t('settings.profilesTab.saveProfile')}
            </button>
          </div>
        </div>
      {/if}

      <div class="space-y-2.5">
        {#each profilesList as prof (prof.id)}
          <div class="flex items-center justify-between p-3.5 bg-neutral-50 dark:bg-neutral-950/60 border border-neutral-200 dark:border-neutral-800 rounded-xl">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-9 h-9 rounded-full bg-cyan-500/10 border border-cyan-500/30 text-cyan-600 dark:text-cyan-400 flex items-center justify-center font-bold text-xs shrink-0">
                {prof.name.slice(0, 2).toUpperCase()}
              </div>
              <div class="min-w-0">
                <h3 class="text-sm font-medium text-neutral-900 dark:text-white truncate">{prof.name}</h3>
                <span class="text-xs {MUTED} capitalize">{prof.role}</span>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span class="text-[11px] px-2.5 py-1 rounded-md bg-neutral-200/70 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 font-mono">
                {prof.has_pin ? t('settings.profilesTab.pinSet') : t('settings.profilesTab.noPin')}
              </span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {:else if activeTab === 'ai'}
    <div class="{CARD} space-y-6">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.ai.title')}</h2>
        <p class="{MUTED} text-sm mt-0.5">{t('settings.ai.subtitle')}</p>
      </div>

      <form onsubmit={handleSaveAi} class="{SUBCARD} space-y-4">
        <div>
          <label for="ai-endpoint" class={LABEL}>{t('settings.ai.endpoint')}</label>
          <input
            id="ai-endpoint"
            type="text"
            bind:value={aiSettings.base_url}
            placeholder="http://localhost:20128/v1"
            class="{INPUT} font-mono"
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label for="ai-key" class={LABEL}>{t('settings.ai.apiKey')}</label>
            <div class="relative">
              <input
                id="ai-key"
                type={showAiApiKey ? 'text' : 'password'}
                bind:value={aiApiKeyInput}
                placeholder={aiSettings.has_api_key ? '•••••••• (Tersimpan)' : 'sk-...'}
                class="{INPUT} font-mono pr-10"
              />
              <button
                type="button"
                onclick={() => (showAiApiKey = !showAiApiKey)}
                class={EYE_BUTTON}
                aria-label={showAiApiKey ? 'Hide API key' : 'Show API key'}
              >
                {#if showAiApiKey}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                {/if}
              </button>
            </div>
          </div>
          <div>
            <label for="ai-model" class={LABEL}>{t('settings.ai.model')}</label>
            <input
              id="ai-model"
              type="text"
              bind:value={aiSettings.model}
              placeholder="ZA126_PRO"
              class="{INPUT} font-mono"
            />
          </div>
        </div>

        <div class="pt-2">
          <label class="flex items-start gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={true}
              class="mt-1 rounded border-neutral-300 dark:border-neutral-700 text-cyan-600 focus:ring-cyan-500"
            />
            <div>
              <span class="text-xs font-semibold text-neutral-900 dark:text-white block">{t('settings.ai.privacyMode')}</span>
              <span class="text-[11px] {MUTED} block mt-0.5">{t('settings.ai.privacyModeBody')}</span>
            </div>
          </label>
        </div>

        <div class="flex justify-end pt-2 gap-2">
          <button
            type="button"
            disabled={isTestingAi}
            class="px-4 py-2 border border-cyan-600/40 hover:bg-cyan-600/5 disabled:opacity-50 text-cyan-700 dark:text-cyan-400 text-xs font-semibold rounded-lg transition-colors cursor-pointer"
            onclick={handleTestAi}
          >
            {isTestingAi ? t('settings.ai.testing') : t('settings.ai.testButton')}
          </button>
          <button
            type="submit"
            disabled={isSavingAi}
            class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-colors shadow-xs cursor-pointer"
          >
            {isSavingAi ? t('settings.ai.saving') : t('settings.ai.saveButton')}
          </button>
        </div>

        {#if aiTestResult}
          <div class="mt-3 p-3 rounded-lg text-xs border {aiTestResult.ok ? 'bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300' : 'bg-red-50 dark:bg-red-950/40 border-red-200 dark:border-red-900 text-red-700 dark:text-red-300'}">
            {aiTestResult.message}
          </div>
        {/if}
      </form>
    </div>
  {:else if activeTab === 'backup'}
    <div class="{CARD} space-y-8">
      <div>
        <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">{t('settings.backup.title')}</h2>
        <p class="{MUTED} text-sm mt-0.5">{t('settings.backup.subtitle')}</p>
      </div>

      <!-- Export Encrypted Backup -->
      <div class="space-y-4">
        <div>
          <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.backup.exportTitle')}</h3>
          <p class="text-xs {MUTED} mt-0.5">{t('settings.backup.exportBody')}</p>
        </div>

        <form onsubmit={handleExportBackup} class="max-w-md space-y-4 {SUBCARD}">
          <div>
            <label for="backup-export-pass" class={LABEL}>{t('settings.backup.exportPassphrase')}</label>
            <div class="relative">
              <input
                id="backup-export-pass"
                type={showExportPassphrase ? 'text' : 'password'}
                minlength="8"
                required
                bind:value={exportPassphrase}
                placeholder="••••••••••••"
                class="{INPUT} pr-10"
              />
              <button
                type="button"
                onclick={() => (showExportPassphrase = !showExportPassphrase)}
                class={EYE_BUTTON}
                aria-label={showExportPassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
              >
                {#if showExportPassphrase}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                {/if}
              </button>
            </div>
          </div>

          <button
            type="submit"
            disabled={isExporting || exportPassphrase.length < 8}
            class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-colors shadow-xs cursor-pointer"
          >
            {isExporting ? t('settings.backup.exporting') : t('settings.backup.exportButton')}
          </button>
        </form>
      </div>

      <hr class="border-neutral-200 dark:border-neutral-800" />

      <!-- Restore Encrypted Backup -->
      <div class="space-y-4">
        <div>
          <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('settings.backup.restoreTitle')}</h3>
          <p class="text-xs {MUTED} mt-0.5">{t('settings.backup.restoreBody')}</p>
        </div>

        <form onsubmit={handleImportBackup} class="max-w-md space-y-4 {SUBCARD}">
          <div>
            <label for="backup-file-input" class={LABEL}>Backup Archive (.cafbackup)</label>
            <input
              id="backup-file-input"
              type="file"
              accept=".cafbackup,.bin,.enc"
              onchange={handleFileSelected}
              class="{INPUT} file:mr-3 file:py-1 file:px-2.5 file:rounded-md file:border-0 file:text-xs file:font-semibold file:bg-cyan-500/10 file:text-cyan-600 dark:file:text-cyan-400 file:cursor-pointer cursor-pointer"
            />
          </div>

          <div>
            <label for="backup-restore-pass" class={LABEL}>{t('settings.backup.restorePassphrase')}</label>
            <div class="relative">
              <input
                id="backup-restore-pass"
                type={showRestorePassphrase ? 'text' : 'password'}
                required
                minlength="8"
                bind:value={restorePassphrase}
                placeholder="••••••••••••"
                class="{INPUT} pr-10"
              />
              <button
                type="button"
                onclick={() => (showRestorePassphrase = !showRestorePassphrase)}
                class={EYE_BUTTON}
                aria-label={showRestorePassphrase ? t('settings.backup.hidePassphrase') : t('settings.backup.showPassphrase')}
              >
                {#if showRestorePassphrase}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                {/if}
              </button>
            </div>
          </div>

          <button
            type="submit"
            disabled={isRestoring || !selectedBackupFile || restorePassphrase.length < 8}
            class="px-4 py-2 bg-amber-600 hover:bg-amber-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-colors shadow-xs cursor-pointer"
          >
            {isRestoring ? t('settings.backup.restoring') : t('settings.backup.restoreButton')}
          </button>
        </form>
      </div>
    </div>
  {:else if activeTab === 'about'}
    <div class="{CARD} space-y-6">
      <div class="flex items-center gap-3.5">
        <div class="p-2.5 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-600 dark:text-cyan-400">
          <Logo size={36} mode="brand" />
        </div>
        <div>
          <h2 class="text-lg font-semibold text-neutral-900 dark:text-white">CAMark Studio</h2>
          <p class="{MUTED} text-xs mt-0.5">{t('settings.about.subtitle')}</p>
        </div>
      </div>

      <div class="{SUBCARD} divide-y divide-neutral-200 dark:divide-neutral-800 text-xs">
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>{t('settings.about.version')}</span>
          <span class="font-mono font-semibold text-cyan-600 dark:text-cyan-400">v{APP_VERSION}</span>
        </div>
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>{t('settings.about.stack')}</span>
          <span class="text-neutral-800 dark:text-neutral-200">{t('settings.about.stackVal')}</span>
        </div>
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>{t('settings.about.license')}</span>
          <span class="text-neutral-800 dark:text-neutral-200">{t('settings.about.licenseVal')}</span>
        </div>
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>{t('settings.about.author')}</span>
          <a href={AUTHOR_URL} target="_blank" rel="noreferrer" class="font-medium text-cyan-600 dark:text-cyan-400 hover:underline">
            {AUTHOR_NAME}
          </a>
        </div>
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>Website</span>
          <a href={WEBSITE_URL} target="_blank" rel="noreferrer" class="font-mono text-cyan-600 dark:text-cyan-400 hover:underline">
            {WEBSITE_URL}
          </a>
        </div>
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>Contact & Support</span>
          <div class="flex items-center gap-3">
            <a href="mailto:{CONTACT_EMAIL}" class="text-neutral-800 dark:text-neutral-200 hover:text-cyan-500 font-mono">
              {CONTACT_EMAIL}
            </a>
            <span class="text-neutral-300 dark:text-neutral-700">•</span>
            <span class="text-neutral-800 dark:text-neutral-200 font-mono">{CONTACT_PHONE}</span>
          </div>
        </div>
        <div class="flex justify-between items-center py-2.5">
          <span class={MUTED}>Studio / Incubator</span>
          <a href={INCUBATOR_URL} target="_blank" rel="noreferrer" class="text-neutral-800 dark:text-neutral-200 hover:text-cyan-500 font-medium">
            {INCUBATOR_NAME} ({INCUBATOR_URL})
          </a>
        </div>
      </div>
    </div>
  {/if}
</div>
