export function lazyThemeTransitionSetup() {
	// 最初は一瞬でテーマが反映させるようにする。
	setTimeout(() => {
		document.documentElement.style.transition = "background-color 0.2s linear";
	}, 100);
}
