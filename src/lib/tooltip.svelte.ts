import { t } from "./i18n.svelte";
import { TIP_IMPACT, type TipData, type TipKey } from "./tips";

export const TIP_ID = "fp-tip";
const DELAY_MS = 400;

interface Shown {
	data: TipData;
	rect: DOMRect;
	owner: HTMLElement;
}

class TipState {
	current = $state<Shown | null>(null);

	hide(): void {
		this.current?.owner.removeAttribute("aria-describedby");
		this.current = null;
	}
}

export const tipState = new TipState();

/** Подсказка по ключу: заголовок — подпись самого контрола, текст и «цена» — из словаря и TIP_IMPACT. */
export function tipFor(key: TipKey, title: string): TipData {
	return { title, body: t(`tip.${key}`), ...TIP_IMPACT[key] };
}

let timer: ReturnType<typeof setTimeout> | undefined;

/** `use:tip={data}` — наведение (с задержкой) или фокус с клавиатуры показывает подсказку. */
export function tip(node: HTMLElement, initial: TipData | undefined) {
	let data = initial;

	const show = (delay: number) => {
		clearTimeout(timer);
		if (!data) return;
		timer = setTimeout(() => {
			if (!data) return;
			tipState.hide();
			tipState.current = { data, rect: node.getBoundingClientRect(), owner: node };
			node.setAttribute("aria-describedby", TIP_ID);
		}, delay);
	};
	const hide = () => {
		clearTimeout(timer);
		if (tipState.current?.owner === node) tipState.hide();
	};
	const onEnter = () => show(DELAY_MS);
	const onFocus = (e: FocusEvent) => {
		if ((e.target as HTMLElement).matches(":focus-visible")) show(0);
	};
	const onKey = (e: KeyboardEvent) => {
		if (e.key === "Escape") hide();
	};

	node.addEventListener("pointerenter", onEnter);
	node.addEventListener("pointerleave", hide);
	node.addEventListener("focusin", onFocus);
	node.addEventListener("focusout", hide);
	node.addEventListener("keydown", onKey);

	return {
		update(next: TipData | undefined) {
			data = next;
			if (next && tipState.current?.owner === node) tipState.current = { ...tipState.current, data: next };
		},
		destroy() {
			hide();
			node.removeEventListener("pointerenter", onEnter);
			node.removeEventListener("pointerleave", hide);
			node.removeEventListener("focusin", onFocus);
			node.removeEventListener("focusout", hide);
			node.removeEventListener("keydown", onKey);
		}
	};
}
