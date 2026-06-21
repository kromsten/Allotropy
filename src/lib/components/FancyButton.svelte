<!-- FancyButton.svelte -->
<script lang="ts">
	const {
		text,
		variant = 'primary',
		size = 'base',
        onclick = () => {}
	}: {
		text: string;
		variant?: 'primary' | 'secondary' | 'premium';
		size?: 'sm' | 'base' | 'lg';
		onclick?: () => void;
	} = $props();

	const premiumStyles = 'shadow-lg hover:shadow-xl transition-all';
    let styles = $derived(` ${variant == "premium" ? premiumStyles : ''}`)
</script>

<button {onclick} 
	class="btn btn-{size} preset-{variant == "secondary" ? 'tonal-secondary' : 'filled-primary'} {variant == "premium" ? premiumStyles : ''}"
>{text}</button>


<!-- 
<button
	type="button"
	onclick={onclick}
	class="btn 
		{variant === 'primary' ? 'preset-filled-primary' : ''}
		{variant === 'secondary' ? 'preset-tonal-secondary' : ''}
		{variant === 'premium' ? 'preset-filled-primary shadow-lg hover:shadow-xl transition-all' : ''}
		{size === 'sm' ? 'btn-sm' : ''}
		{size === 'base' ? 'btn-base' : ''}
		{size === 'lg' ? 'btn-lg' : ''}"
>
	{text}
</button> -->