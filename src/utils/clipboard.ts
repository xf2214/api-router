/** 复制文本：优先 navigator.clipboard，失败降级 execCommand。全失败返回 false。 */
export async function writeTextWithFallback(text: string): Promise<boolean> {
  try {
    const nav = navigator as Navigator & { clipboard?: { writeText(t: string): Promise<void> } };
    if (nav.clipboard) { await nav.clipboard.writeText(text); return true; }
  } catch { /* fall through */ }
  try {
    const ta = document.createElement('textarea');
    ta.value = text; ta.setAttribute('readonly', '');
    ta.style.position = 'fixed'; ta.style.opacity = '0';
    document.body.appendChild(ta);
    let ok = false;
    try {
      ta.select();
      ok = document.execCommand('copy');
    } finally {
      if (document.body.contains(ta)) document.body.removeChild(ta);
    }
    return ok;
  } catch { return false; }
}
