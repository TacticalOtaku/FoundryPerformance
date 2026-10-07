import { untrack } from "svelte";
import type { Attachment } from "svelte/attachments";
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

const sameTip = (a: TipData, b: TipData) =>
	a.title === b.title &&
	a.body === b.body &&
	a.fps === b.fps &&
	a.look === b.look &&
	a.lines?.join("\n") === b.lines?.join("\n") &&
	JSON.stringify(a.checks) === JSON.stringify(b.checks);

/**
 * `{@attach tip(() => data)}` — наведение (с задержкой) или фокус с клавиатуры показывает подсказку.
 * Данные берутся через функцию: сама привязка не пересоздаётся, когда они меняются,
 * а уже открытая подсказка обновляет текст на месте.
 */
export function tip(get: () => TipData | undefined): Attachment<HTMLElement> {
	return (node) => {
		const show = (delay: number) => {
			clearTimeout(timer);
			timer = setTimeout(() => {
				const data = get();
				if (!data) return;
				tipState.hide();
				tipState.current = { data, rect: node.getBoundingClientRect(), owner: node };
				node.setAttribute("aria-describedby", TIP_ID);
			}, delay);
		};
		const hide = () => {
			clearTimeout(timer);
			// Если элемент удаляют из DOM, focusout приходит прямо во время разборки,
			// а менять $state в этот момент Svelte запрещает — прячем на следующем тике.
			if (tipState.current?.owner === node) {
				queueMicrotask(() => {
					if (tipState.current?.owner === node) tipState.hide();
				});
			}
		};
		const onEnter = () => show(DELAY_MS);
		const onFocus = (e: FocusEvent) => {
			if ((e.target as HTMLElement).matches(":focus-visible")) show(0);
		};
		const onKey = (e: KeyboardEvent) => {
			if (e.key === "Escape") hide();
		};

		// Эффект следит только за данными подсказки; открыта ли она сейчас — читаем без подписки,
		// иначе запись tipState.current перезапускала бы этот же эффект.
		$effect(() => {
			const next = get();
			untrack(() => {
				const cur = tipState.current;
				if (next && cur?.owner === node && !sameTip(cur.data, next)) tipState.current = { ...cur, data: next };
			});
		});

		node.addEventListener("pointerenter", onEnter);
		node.addEventListener("pointerleave", hide);
		node.addEventListener("focusin", onFocus);
		node.addEventListener("focusout", hide);
		node.addEventListener("keydown", onKey);

		return () => {
			hide();
			node.removeEventListener("pointerenter", onEnter);
			node.removeEventListener("pointerleave", hide);
			node.removeEventListener("focusin", onFocus);
			node.removeEventListener("focusout", hide);
			node.removeEventListener("keydown", onKey);
		};
	};
}
