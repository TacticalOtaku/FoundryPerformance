const ID = "fp-no-blur";
const CSS = "*,*::before,*::after{backdrop-filter:none!important;-webkit-backdrop-filter:none!important}";

export function setUiBlur(doc: Document, enabled: boolean): void {
	const existing = doc.getElementById(ID);
	if (enabled) {
		existing?.remove();
		return;
	}
	if (existing) return;
	const style = doc.createElement("style");
	style.id = ID;
	style.textContent = CSS;
	(doc.head ?? doc.documentElement).appendChild(style);
}
