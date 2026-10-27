import {
  cmdGetAiSettings,
  cmdSaveAiSettings,
  cmdSetAiApiKey,
  cmdClearAiApiKey,
  cmdAiPreviewContext,
  cmdAiChat,
  cmdTestAiConnection,
  type AiSettings,
  type AiChatResponse,
  type ContextPayload,
  type PrivacyLevel,
  type DocContext,
} from '$lib/generated/commands';

export type { AiSettings, AiChatResponse, ContextPayload, PrivacyLevel, DocContext };

export async function getAiSettings(): Promise<AiSettings> {
  return await cmdGetAiSettings();
}

export async function saveAiSettings(settings: AiSettings, apiKey?: string): Promise<void> {
  await cmdSaveAiSettings({ settings });
  if (apiKey !== undefined) {
    if (apiKey.trim().length > 0) {
      await cmdSetAiApiKey({ apiKey: apiKey.trim() });
    } else {
      await cmdClearAiApiKey();
    }
  }
}

export async function setAiApiKey(apiKey: string): Promise<void> {
  await cmdSetAiApiKey({ apiKey });
}

export async function clearAiApiKey(): Promise<void> {
  await cmdClearAiApiKey();
}

export async function previewAiContext(level: PrivacyLevel): Promise<ContextPayload> {
  return await cmdAiPreviewContext({ level });
}

export async function chatWithAi(
  prompt: string,
  docContext?: DocContext | null,
  privacyLevel?: PrivacyLevel | null,
  consentGiven?: boolean | null
): Promise<AiChatResponse> {
  return await cmdAiChat({
    prompt,
    docContext: docContext ?? undefined,
    privacyLevel: privacyLevel ?? undefined,
    consentGiven: consentGiven ?? undefined,
  });
}

export async function testAiConnection(
  baseUrl: string,
  model: string,
  apiKey?: string
): Promise<string> {
  return await cmdTestAiConnection({ baseUrl, model, apiKey: apiKey ?? null });
}
