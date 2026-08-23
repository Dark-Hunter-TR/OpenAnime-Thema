<script lang="ts">
	import { Button, IconButton, Slider, TextBlock, TextBox, ToggleSwitch } from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";

	import ColorField from "$lib/ColorField.svelte";
	import Icon from "$lib/Icon.svelte";
	import StatusBar from "$lib/StatusBar.svelte";
	import Section from "$lib/Section.svelte";
	import SelectField from "$lib/SelectField.svelte";
	import { CARD_TOKENS, FONT_PRESETS, MASCOT_SLOTS, TEXT_TOKENS } from "$lib/advanced";
	import { LINK_TOKENS, SURFACE_TOKENS, SYSTEM_TOKENS } from "$lib/customization";
	import {
		BADGE_INDEPENDENT_FIELDS,
		SURFACE_OWNED_TOKENS,
		reseedSection,
		resetAdvSection,
		type AdvSection,
		type AdvState
	} from "$lib/advancedBuild";

	export let adv: AdvState;
	/** Görsel seçtirip data URI döndüren fonksiyon (Tauri dosya seçici). */
	export let pickImage: () => Promise<string | null>;
	/**
	 * Sıfırlamanın ve kapalı bölümlerin TABANI: düzenlenen şeyin "orijinali".
	 *
	 * Yeni bir tema oluşturuluyorsa sitenin kendi değerleri; bir `.css` dosyası
	 * ya da GitHub içeriği açıldıysa O TEMANIN içe aktarma anındaki değerleri.
	 * Üst bileşen hesaplıyor (`+page.svelte` -> `advBaseline`).
	 *
	 * Burada `mode`/`ramp` alıp `defaultAdv(mode, ramp)` çağırmak yetmiyordu:
	 * o her zaman SİTE varsayılanını verir. İçe aktarılmış bir temada bir
	 * bölümü kapatıp açmak, temanın değerlerini site varsayılanlarıyla
	 * değiştiriyordu — kullanıcı yalnızca aç-kapa yapıyor, logosunun yazı tipi
	 * ve rengi gidiyordu. Taban artık dışarıdan geliyor, o iki prop da gerekmiyor.
	 */
	export let baseline: AdvState;
	/**
	 * Hangi bölümlerin gösterileceğini söyleyen süzgeç.
	 *
	 * Bölüm listesi burada değil ÜST BİLEŞENDE tutuluyor: "Temel" sekmesinin
	 * hangi beş ayarı gösterdiği bir ürün kararı ve o karar, bu bileşenin
	 * bilmediği (vurgu rengi, köşe yumuşaklığı gibi) bölümleri de kapsıyor.
	 */
	export let show: (section: string) => boolean = () => true;

	/**
	 * Bölümlerin üstüne grup başlıkları yazılsın mı.
	 *
	 * Yalnızca "Tümü" sekmesinde açık: orada yirmiden fazla bölüm tek akışta
	 * duruyor ve başlıksız bir liste okunmuyor. "Temel"de zaten beş bölüm var,
	 * başlık gürültüden ibaret olurdu.
	 */
	export let grouped = false;

	let busy = "";

	async function chooseInto(apply: (uri: string) => void, key: string) {
		busy = key;
		try {
			const uri = await pickImage();
			if (uri) apply(uri);
		} finally {
			busy = "";
		}
	}

	/** Bölüm başlığındaki sıfırlama düğmesi. */
	const reset = (section: AdvSection) => () => {
		adv = resetAdvSection(adv, section, baseline);
	};

	// --- Varsayılanları taze tut ---------------------------------------------
	// Bölüm KAPALIYKEN değerleri TABANDAN yeniden tohumla. Açıkken
	// dokunmuyoruz, yoksa kullanıcının girdiği değerler silinirdi.
	//
	// Taban, düzenlenen şeyin "orijinali": yeni bir tema oluşturuluyorsa
	// sitenin kendi değerleri, bir dosya ya da GitHub içeriği açıldıysa O
	// TEMANIN değerleri (bkz. `+page.svelte` -> `advBaseline`).
	//
	// Eskiden burada her zaman `defaultAdv(mode, ramp)` vardı, yani site
	// varsayılanları. İçe aktarılmış bir temada bir bölümü kapatıp açmak
	// temanın değerlerini site varsayılanlarıyla değiştiriyordu: kullanıcı
	// yalnızca aç-kapa yapıyor, logosunun yazı tipi ve rengi gidiyordu.
	$: if (!adv.text.on) adv.text.colors = structuredClone(baseline.text.colors);
	$: if (!adv.cards.on) adv.cards.colors = structuredClone(baseline.cards.colors);
	$: if (!adv.surface.on) adv.surface.colors = structuredClone(baseline.surface.colors);
	$: if (!adv.links.on) adv.links.colors = structuredClone(baseline.links.colors);
	$: if (!adv.system.on) adv.system.colors = structuredClone(baseline.system.colors);

	$: if (!adv.sidebar.on) adv.sidebar = structuredClone(baseline.sidebar);
	// Rozetlerde gizleme ve yazı, renk anahtarına BAĞLI DEĞİL; tazelemede
	// korunuyorlar (gerekçe: `advancedBuild.ts` -> `reseedSection`).
	$: if (!adv.badges.on)
		adv.badges = reseedSection(adv.badges, baseline.badges, BADGE_INDEPENDENT_FIELDS);
	$: if (!adv.banner.on) adv.banner = structuredClone(baseline.banner);
	$: if (!adv.scrollbar.on) adv.scrollbar = structuredClone(baseline.scrollbar);
	$: if (!adv.comments.on) adv.comments = structuredClone(baseline.comments);

	$: sizedMascots = MASCOT_SLOTS.filter((s) => s.size !== null);
</script>

{#if grouped}<h3 class="group">Genel görünüm</h3>{/if}

<slot name="accent" />

{#if show("text")}
<Section icon="focus" title="Yazı renkleri" onReset={reset("text")}>
	<ToggleSwitch bind:checked={adv.text.on}>Metin ve odak renklerini özelleştir</ToggleSwitch>
	{#each TEXT_TOKENS as spec, i}
		<ColorField
			{spec}
			bind:hex={adv.text.colors[i].hex}
			bind:alpha={adv.text.colors[i].alpha}
			disabled={!adv.text.on}
		/>
	{/each}
</Section>
{/if}

{#if show("surface")}
<Section icon="surface" title="Sayfa arka planı" onReset={reset("surface")}>
	<TextBlock variant="caption">
		Sayfa zemini, panel yüzeyleri ve katmanların renk tonları.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.surface.on}>Yüzey renklerini özelleştir</ToggleSwitch>
	{#each SURFACE_TOKENS as spec, i}
		<ColorField
			{spec}
			bind:hex={adv.surface.colors[i].hex}
			bind:alpha={adv.surface.colors[i].alpha}
			disabled={!adv.surface.on}
		/>
	{/each}
</Section>
{/if}

{#if show("typo")}
<Section icon="typography" title="Yazı tipi" onReset={reset("typo")}>
	<ToggleSwitch bind:checked={adv.typo.on}>Yazı tipini değiştir</ToggleSwitch>
	<TextBlock variant="caption">Hazır seçenekler gelişmiş temalarda kullanılanlardır.</TextBlock>
	<!--
		Kendi açılır listemiz, kütüphanenin `ComboBox`'ı değil: onun listesi bu
		panelde tetikleyicisinden kopup pencerenin köşesine açılıyordu. Gerekçe
		ve kalıp `SelectField.svelte`'te.
	-->
	<SelectField
		items={FONT_PRESETS.map((f, i) => ({ name: f.name, value: i }))}
		bind:value={adv.typo.preset}
		disabled={!adv.typo.on}
		label="Hazır yazı tipi"
	/>
	<div class="field">
		<TextBlock variant="caption">Ya da kendi font-family'niz</TextBlock>
		<TextBox
			bind:value={adv.typo.custom}
			disabled={!adv.typo.on}
			placeholder="'Segoe UI', sans-serif"
			clearButton={false}
		/>
		<TextBlock variant="caption">
			Doldurulursa hazır seçeneğin yerine geçer. Google Fonts kullanacaksanız hazır
			seçeneklerden birini seçin — @import satırı o zaman otomatik eklenir.
		</TextBlock>
	</div>

	<ToggleSwitch bind:checked={adv.typo.sizeOn} disabled={!adv.typo.on}>
		Yazı boyutlarını ölçekle
	</ToggleSwitch>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Ölçek — {adv.typo.scale.toFixed(2)}×</TextBlock>
		<Slider
			bind:value={adv.typo.scale}
			min={0.8}
			max={1.5}
			step={0.05}
			disabled={!adv.typo.on || !adv.typo.sizeOn}
			suffix="×"
		/>
	</label>
	<TextBlock variant="caption">
		Sitenin kendi <code>--fds-*-font-size</code> token'ları çarpılır; yeni bir
		tipografi ölçeği eklenmez. 1.00× = sitenin orijinal boyutları.
	</TextBlock>
</Section>
{/if}

<slot name="shape" />



{#if grouped}<h3 class="group">Görseller</h3>{/if}

{#if show("bg")}
<Section icon="background" title="Arka plan görseli" onReset={reset("bg")}>
	<TextBlock variant="caption">
		Görsel sabit bir katmana basılır ve üstteki yüzeyler şeffaflaştırılır,
		böylece arkaplan sayfa boyunca görünür kalır.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.bg.on}>Arkaplan görseli kullan</ToggleSwitch>
	<div class="row">
		<Button disabled={!adv.bg.on} on:click={() => chooseInto((u) => (adv.bg.dataUri = u), "bg")}>
			<Icon name="open" size={14} /><span class="gap">Görsel seç…</span>
		</Button>
		<TextBox bind:value={adv.bg.dataUri} disabled={!adv.bg.on} placeholder="veya görsel URL'si girin (https://...)" clearButton={false} />
		{#if adv.bg.dataUri}
			<Tooltip text="Kaldır">
				<IconButton on:click={() => (adv.bg.dataUri = "")}>
					<Icon name="reset" size={14} />
				</IconButton>
			</Tooltip>
		{/if}
	</div>
	{#if adv.bg.dataUri}
		<img class="preview" src={adv.bg.dataUri} alt="Arkaplan önizlemesi" />
	{/if}
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Karartma — %{adv.bg.dim}</TextBlock>
		<Slider bind:value={adv.bg.dim} min={0} max={90} step={1} disabled={!adv.bg.on} suffix="%" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Bulanıklık — {adv.bg.blur}px</TextBlock>
		<Slider bind:value={adv.bg.blur} min={0} max={20} step={1} disabled={!adv.bg.on} suffix="px" />
	</label>
</Section>
{/if}

{#if show("logo")}
<Section icon="brand" title="Logo ve site adı" onReset={reset("logo")}>
	<TextBlock variant="caption">
		Sitenin orijinal logosu ve adı gizlenir; yerine buradan seçtiğiniz görsel
		ve metin gösterilir.
	</TextBlock>
	<StatusBar
		severity="information"
		title="Bağımsız"
		message="Logo görseli ve site adı ayrı ayrı tutulur: yalnızca adı değiştirirseniz logo yerinde kalır. Uzun adlar rozete taşmadan üç noktayla kısalır."
		closable={false}
	/>

	<ToggleSwitch bind:checked={adv.logo.imageOn}>Logo görselini değiştir</ToggleSwitch>
	<div class="row">
		<Button
			disabled={!adv.logo.imageOn}
			on:click={() => chooseInto((u) => (adv.logo.dataUri = u), "logo")}
		>
			<Icon name="open" size={14} /><span class="gap">Görsel seç…</span>
		</Button>
		<TextBox bind:value={adv.logo.dataUri} disabled={!adv.logo.imageOn} placeholder="veya görsel URL'si girin (https://...)" clearButton={false} />
		{#if adv.logo.dataUri}
			<img class="thumb" src={adv.logo.dataUri} alt="Logo önizlemesi" />
			<Tooltip text="Kaldır">
				<IconButton on:click={() => (adv.logo.dataUri = "")}>
					<Icon name="reset" size={14} />
				</IconButton>
			</Tooltip>
		{/if}
	</div>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Logo boyutu — {adv.logo.size}px</TextBlock>
		<Slider bind:value={adv.logo.size} min={12} max={64} step={1} disabled={!adv.logo.imageOn} suffix="px" />
	</label>

	<ToggleSwitch bind:checked={adv.logo.textOn}>Site adı yaz</ToggleSwitch>
	<div class="field">
		<TextBox bind:value={adv.logo.text} disabled={!adv.logo.textOn} placeholder="OpenAnime" clearButton={false} />
	</div>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Yazı boyutu — {adv.logo.textSize}px</TextBlock>
		<Slider bind:value={adv.logo.textSize} min={10} max={32} step={1} disabled={!adv.logo.textOn} suffix="px" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Adın en fazla genişliği — {adv.logo.maxWidth}px</TextBlock>
		<Slider bind:value={adv.logo.maxWidth} min={60} max={400} step={10} disabled={!adv.logo.textOn} suffix="px" />
	</label>
	<TextBlock variant="caption">
		Bu sınırı aşan adlar üç noktayla kısalır; rozetin üstüne taşmaz.
	</TextBlock>

	<ToggleSwitch bind:checked={adv.logo.badgeHidden}>NEXT-GEN rozetini gizle</ToggleSwitch>
	<TextBlock variant="caption">
		Rozet gizlendiğinde site adı için tüm satır boşalır.
	</TextBlock>

	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Logo ile ad arası boşluk — {adv.logo.gap}px</TextBlock>
		<Slider
			bind:value={adv.logo.gap}
			min={0}
			max={32}
			step={1}
			disabled={!adv.logo.imageOn && !adv.logo.textOn}
			suffix="px"
		/>
	</label>
</Section>
{/if}

{#if show("mascot")}
<Section icon="mascot" title="Setsuki karakteri" onReset={reset("mascot")}>
	<TextBlock variant="caption">
		Sitede maskotun beş ayrı örneği var; her birini ayrı değiştirebilirsiniz.
	</TextBlock>

	<ToggleSwitch bind:checked={adv.mascot.sizeOn}>Boyutları ayarla</ToggleSwitch>
	<StatusBar
		severity="information"
		title="Yalnızca sabit ölçülü örnekler"
		message="Genel ve diyalog maskotları sitede akışkan boyutlu (height: 100%, width: auto) — onlara sabit ölçü vermek düzeni bozar, o yüzden kaydırıcıları yok. Diğer üçünün sitedeki gerçek ölçüsü 170, 170 ve 150px."
		closable={false}
	/>
	{#each sizedMascots as slot}
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<TextBlock variant="caption">{slot.label} — {adv.mascot.sizes[slot.id]}px</TextBlock>
			<Slider
				bind:value={adv.mascot.sizes[slot.id]}
				min={40}
				max={400}
				step={5}
				disabled={!adv.mascot.sizeOn}
				suffix="px"
			/>
		</label>
	{/each}

	{#each MASCOT_SLOTS as slot}
		<div class="field">
			<div class="row-between">
				<Tooltip text={slot.selector}>
					<TextBlock variant="caption">{slot.label}</TextBlock>
				</Tooltip>
				{#if adv.mascot.images[slot.id]}
					<img class="thumb" src={adv.mascot.images[slot.id]} alt="{slot.label} önizlemesi" />
				{/if}
			</div>
			<div class="row">
				<Button
					disabled={busy === slot.id}
					on:click={() => chooseInto((u) => (adv.mascot.images[slot.id] = u), slot.id)}
				>
					<Icon name="open" size={14} /><span class="gap">Görsel seç…</span>
				</Button>
				{#if adv.mascot.images[slot.id]}
					<Tooltip text="Varsayılana dön">
						<IconButton
							on:click={() => {
								delete adv.mascot.images[slot.id];
								adv.mascot.images = adv.mascot.images;
							}}
						>
							<Icon name="reset" size={14} />
						</IconButton>
					</Tooltip>
				{/if}
			</div>
			<TextBlock variant="caption">{slot.hint}</TextBlock>
		</div>
	{/each}
</Section>
{/if}

{#if show("avatar")}
<Section icon="avatar" title="Profil fotoğrafı" onReset={reset("avatar")}>
	<TextBlock variant="caption">
		Ayar <strong>yalnızca üst çubuktaki profil görseline</strong> uygulanır. Kart
		içindeki avatarların boyutunu etkilemez. Sitedeki varsayılan değer 32px'tir.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.avatar.on}>Avatar boyutunu ayarla</ToggleSwitch>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Boyut — {adv.avatar.size}px</TextBlock>
		<Slider bind:value={adv.avatar.size} min={20} max={80} step={1} disabled={!adv.avatar.on} suffix="px" />
	</label>
</Section>
{/if}



{#if grouped}<h3 class="group">Sayfa parçaları</h3>{/if}

{#if show("cards")}
<Section icon="card" title="Kartlar" onReset={reset("cards")}>
	<TextBlock variant="caption">
		Kart arkaplanı, köşe yuvarlaklığı, hover'da yükselme ve parıltı gibi
		görünüm ayarlarını buradan özelleştirebilirsiniz.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.cards.on}>Kart görünümünü özelleştir</ToggleSwitch>
	{#each CARD_TOKENS as spec, i}
		<!-- Sahibi "Yüzeyler" olan token burada çizilmiyor: iki bölümden de
		     yazıldığında sonra gelen kazanıyor ve buradaki kontrol hiçbir şey
		     yapmıyordu (gerekçe: `advancedBuild.ts` -> `SURFACE_OWNED_TOKENS`). -->
		{#if !SURFACE_OWNED_TOKENS.has(spec.token)}
			<ColorField
				{spec}
				bind:hex={adv.cards.colors[i].hex}
				bind:alpha={adv.cards.colors[i].alpha}
				disabled={!adv.cards.on}
			/>
		{/if}
	{/each}
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Kart köşe yuvarlaklığı — {adv.cards.radius}px</TextBlock>
		<Slider bind:value={adv.cards.radius} min={0} max={40} step={1} disabled={!adv.cards.on} suffix="px" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Hover'da yükselme — {adv.cards.lift}px</TextBlock>
		<Slider bind:value={adv.cards.lift} min={0} max={16} step={1} disabled={!adv.cards.on} suffix="px" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">
			Kenarlık kalınlığı — {adv.cards.borderWidth === 0 ? "yok (sitenin varsayılanı)" : `${adv.cards.borderWidth}px`}
		</TextBlock>
		<Slider bind:value={adv.cards.borderWidth} min={0} max={6} step={1} disabled={!adv.cards.on} suffix="px" />
	</label>
	<ToggleSwitch bind:checked={adv.cards.glow} disabled={!adv.cards.on}>
		Hover'da parıltı
	</ToggleSwitch>
	<div class="field">
		<TextBlock variant="caption">Parıltı rengi</TextBlock>
		<TextBox bind:value={adv.cards.glowColor} disabled={!adv.cards.on || !adv.cards.glow} clearButton={false} />
	</div>
	<ToggleSwitch bind:checked={adv.cards.maskOn} disabled={!adv.cards.on}>
		Kart görselinin altını silikleştir
	</ToggleSwitch>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Silikleşme başlangıcı — %{adv.cards.maskStart}</TextBlock>
		<Slider
			bind:value={adv.cards.maskStart}
			min={0}
			max={100}
			step={1}
			disabled={!adv.cards.on || !adv.cards.maskOn}
			suffix="%"
		/>
	</label>
	<TextBlock variant="caption">
		Kart görselinin alt kısmındaki silikleşme maskesinin başladığı nokta.
	</TextBlock>
</Section>
{/if}

{#if show("banner")}
<Section icon="banner" title="Ana sayfa üst bölümü" onReset={reset("banner")}>
	<TextBlock variant="caption">
		Ana sayfadaki kayan kartların çerçeve rengini ve ilerleme çubuğunun
		görünümünü buradan özelleştirebilirsiniz.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.banner.on}>Banner'ı özelleştir</ToggleSwitch>
	<div class="field">
		<TextBlock variant="caption">Seçili kartın çerçeve rengi</TextBlock>
		<TextBox bind:value={adv.banner.outlineColor} disabled={!adv.banner.on} clearButton={false} />
	</div>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">İlerleme çubuğu kalınlığı — {adv.banner.progressHeight}px</TextBlock>
		<Slider bind:value={adv.banner.progressHeight} min={1} max={16} step={1} disabled={!adv.banner.on} suffix="px" />
	</label>
	<div class="field">
		<TextBlock variant="caption">İlerleme çubuğu rengi</TextBlock>
		<TextBox bind:value={adv.banner.progressColor} disabled={!adv.banner.on} clearButton={false} />
	</div>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">İlerleme çubuğu yuvarlaklığı — {adv.banner.progressRadius}px</TextBlock>
		<Slider bind:value={adv.banner.progressRadius} min={0} max={50} step={1} disabled={!adv.banner.on} suffix="px" />
	</label>
</Section>
{/if}

{#if show("sidebar")}
<Section icon="sidebar" title="Kenar çubuğu ve menü" onReset={reset("sidebar")}>
	<TextBlock variant="caption">
		Kenar çubuğunun genişliğini, seçili öğenin dolgu rengini ve seçim
		göstergesinin görünümünü buradan özelleştirebilirsiniz.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.sidebar.on}>Kenar çubuğunu özelleştir</ToggleSwitch>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Genişlik — {adv.sidebar.width}px</TextBlock>
		<Slider bind:value={adv.sidebar.width} min={48} max={200} step={1} disabled={!adv.sidebar.on} suffix="px" />
	</label>
	<ColorField
		spec={{
			token: "--fds-control-solid-fill-default",
			label: "Seçili öğe dolgusu",
			hint: ".sidebar a.selected'in arkaplanı",
			alpha: true,
			defaultAlpha: 100
		}}
		bind:hex={adv.sidebar.selected.hex}
		bind:alpha={adv.sidebar.selected.alpha}
		disabled={!adv.sidebar.on}
	/>
	<TextBlock variant="bodyStrong">Seçim göstergesi</TextBlock>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Kalınlık — {adv.sidebar.indicatorWidth}px</TextBlock>
		<Slider bind:value={adv.sidebar.indicatorWidth} min={0} max={10} step={1} disabled={!adv.sidebar.on} suffix="px" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Uzunluk — {adv.sidebar.indicatorHeight}px</TextBlock>
		<Slider bind:value={adv.sidebar.indicatorHeight} min={4} max={40} step={1} disabled={!adv.sidebar.on} suffix="px" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Yuvarlaklık — {adv.sidebar.indicatorRadius}px</TextBlock>
		<Slider bind:value={adv.sidebar.indicatorRadius} min={0} max={10} step={1} disabled={!adv.sidebar.on} suffix="px" />
	</label>
	<div class="field">
		<TextBlock variant="caption">Gösterge rengi</TextBlock>
		<TextBox bind:value={adv.sidebar.indicatorColor} disabled={!adv.sidebar.on} clearButton={false} />
	</div>
</Section>
{/if}

{#if show("player")}
<Section icon="player" title="Video oynatıcı" onReset={reset("player")}>
	<TextBlock variant="caption">
		Önizlemede bir bölüm sayfası açıkken değişiklikler anında görünür.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.player.on}>Oynatıcıyı özelleştir</ToggleSwitch>

	<TextBlock variant="bodyStrong">Kontrol çubuğu</TextBlock>
	<ColorField
		spec={{
			token: ".bottom-controls",
			label: "Çubuk arkaplanı",
			hint: "Alt kontrol çubuğu — sitede kendi arkaplanı yok, %0 = dokunma",
			alpha: true,
			defaultAlpha: 0
		}}
		bind:hex={adv.player.barBg.hex}
		bind:alpha={adv.player.barBg.alpha}
		disabled={!adv.player.on}
	/>
	<div class="field">
		<TextBlock variant="caption">İkon rengi</TextBlock>
		<TextBox bind:value={adv.player.iconColor} disabled={!adv.player.on} clearButton={false} />
	</div>

	<TextBlock variant="bodyStrong">İlerleme çubuğu</TextBlock>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Kalınlık — {adv.player.progressHeight}px</TextBlock>
		<Slider
			bind:value={adv.player.progressHeight}
			min={2}
			max={16}
			step={1}
			disabled={!adv.player.on}
			suffix="px"
		/>
	</label>
	<ColorField
		spec={{
			token: ".slider-rail",
			label: "Zemin rengi",
			hint: "Dolmamış kısım — sitede --fds-control-strong-fill-default",
			alpha: true,
			defaultAlpha: 54
		}}
		bind:hex={adv.player.railColor.hex}
		bind:alpha={adv.player.railColor.alpha}
		disabled={!adv.player.on}
	/>
	<div class="field">
		<TextBlock variant="caption">Dolgu rengi</TextBlock>
		<TextBox bind:value={adv.player.trackColor} disabled={!adv.player.on} clearButton={false} />
	</div>
	<div class="field">
		<TextBlock variant="caption">Topuz rengi</TextBlock>
		<TextBox bind:value={adv.player.thumbColor} disabled={!adv.player.on} clearButton={false} />
	</div>
	<ToggleSwitch bind:checked={adv.player.glow} disabled={!adv.player.on}>
		Dolguya parıltı
	</ToggleSwitch>

	<TextBlock variant="bodyStrong">Bölüm listesi</TextBlock>
	<ColorField
		spec={{
			token: ".player-episode-list-item",
			label: "Bölüm kartı arkaplanı",
			hint: "Sitede 11 kural",
			alpha: true,
			defaultAlpha: 5
		}}
		bind:hex={adv.player.episodeBg.hex}
		bind:alpha={adv.player.episodeBg.alpha}
		disabled={!adv.player.on}
	/>
	<div class="field">
		<TextBlock variant="caption">Oynayan bölümün vurgusu</TextBlock>
		<TextBox bind:value={adv.player.currentColor} disabled={!adv.player.on} clearButton={false} />
	</div>

	<TextBlock variant="bodyStrong">Altyazı</TextBlock>
	<StatusBar
		severity="caution"
		title="Sınırlı destek"
		message="Site altyazıyı iki yolla çiziyor: native track (video::cue — CSS işler) ve ASS/SSA için <canvas> (CSS işlemez). Bu ayarlar yalnızca native track kullanan bölümlerde etkili olur."
		closable={false}
	/>
	<ToggleSwitch bind:checked={adv.player.cueOn} disabled={!adv.player.on}>
		Altyazı stilini değiştir
	</ToggleSwitch>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Yazı boyutu — {adv.player.cueSize}px</TextBlock>
		<Slider
			bind:value={adv.player.cueSize}
			min={10}
			max={48}
			step={1}
			disabled={!adv.player.on || !adv.player.cueOn}
			suffix="px"
		/>
	</label>
	<div class="field">
		<TextBlock variant="caption">Yazı rengi</TextBlock>
		<TextBox
			bind:value={adv.player.cueColor}
			disabled={!adv.player.on || !adv.player.cueOn}
			clearButton={false}
		/>
	</div>
	<ToggleSwitch bind:checked={adv.player.cueOutline} disabled={!adv.player.on || !adv.player.cueOn}>
		Siyah kontur (sitenin varsayılanı)
	</ToggleSwitch>
</Section>
{/if}

{#if show("comments")}
<Section icon="comments" title="Yorumlar" onReset={reset("comments")}>
	<TextBlock variant="caption">
		Yorum kartlarının arkaplanını, köşe yuvarlaklığını ve giriş kutusunun
		odaklanma rengini buradan ayarlayabilirsiniz.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.comments.on}>Yorumları özelleştir</ToggleSwitch>
	<ColorField
		spec={{
			token: ".comment",
			label: "Yorum arkaplanı",
			hint: "Sitede yorumun kendi arkaplanı yok; kart yüzeyi temel alındı",
			alpha: true,
			defaultAlpha: 3
		}}
		bind:hex={adv.comments.bg.hex}
		bind:alpha={adv.comments.bg.alpha}
		disabled={!adv.comments.on}
	/>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Köşe yuvarlaklığı — {adv.comments.radius}px</TextBlock>
		<Slider bind:value={adv.comments.radius} min={0} max={24} step={1} disabled={!adv.comments.on} suffix="px" />
	</label>
	<div class="field">
		<TextBlock variant="caption">Giriş kutusu odak rengi</TextBlock>
		<TextBox bind:value={adv.comments.focusColor} disabled={!adv.comments.on} clearButton={false} />
	</div>
</Section>
{/if}

{#if show("badges")}
<Section icon="badge" title="Rozetler" onReset={reset("badges")}>
	<TextBlock variant="caption">
		Her rozeti ayrı ayrı gizleyebilir, yazısını değiştirebilir ve renklerini
		özelleştirebilirsiniz. Varsayılanlar sitenin kendi gradyanlarıdır;
		"NEXT-GEN" rozeti sitede vurgu renginden türediği için varsayılanı da
		seçili vurguyla birlikte değişir.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.badges.on}>Rozet renklerini özelleştir</ToggleSwitch>

	<TextBlock variant="bodyStrong">NEXT-GEN rozeti</TextBlock>
	<!--
		Gizleme ve yazı, renk anahtarından BAĞIMSIZ: rozeti kaldırmak ya da
		yazısını değiştirmek isteyen biri renklerini de özelleştirmek zorunda
		kalmamalı.
	-->
	<ToggleSwitch bind:checked={adv.badges.badgeHidden}>Rozeti gizle</ToggleSwitch>
	<div class="field">
		<TextBlock variant="caption">Yazı (boşsa sitenin kendi yazısı)</TextBlock>
		<TextBox
			bind:value={adv.badges.badgeText}
			disabled={adv.badges.badgeHidden}
			placeholder="NEXT-GEN"
			clearButton={false}
		/>
	</div>
	<div class="row2">
		<div class="field">
			<TextBlock variant="caption">Başlangıç</TextBlock>
			<TextBox bind:value={adv.badges.badgeFrom} disabled={!adv.badges.on} clearButton={false} />
		</div>
		<div class="field">
			<TextBlock variant="caption">Bitiş</TextBlock>
			<TextBox bind:value={adv.badges.badgeTo} disabled={!adv.badges.on} clearButton={false} />
		</div>
	</div>

	<TextBlock variant="bodyStrong">Yayınlandı rozeti</TextBlock>
	<ToggleSwitch bind:checked={adv.badges.releasedHidden}>Rozeti gizle</ToggleSwitch>
	<div class="field">
		<TextBlock variant="caption">Yazı (boşsa sitenin kendi yazısı)</TextBlock>
		<TextBox
			bind:value={adv.badges.releasedText}
			disabled={adv.badges.releasedHidden}
			placeholder="Yayınlandı"
			clearButton={false}
		/>
	</div>
	<div class="row2">
		<div class="field">
			<TextBlock variant="caption">Başlangıç</TextBlock>
			<TextBox bind:value={adv.badges.releasedFrom} disabled={!adv.badges.on} clearButton={false} />
		</div>
		<div class="field">
			<TextBlock variant="caption">Bitiş</TextBlock>
			<TextBox bind:value={adv.badges.releasedTo} disabled={!adv.badges.on} clearButton={false} />
		</div>
	</div>

	<TextBlock variant="bodyStrong">Geliştirilmiş şeridi</TextBlock>
	<ToggleSwitch bind:checked={adv.badges.enhancedHidden}>Şeridi gizle</ToggleSwitch>
	<div class="field">
		<TextBlock variant="caption">Yazı (boşsa sitenin kendi yazısı)</TextBlock>
		<TextBox
			bind:value={adv.badges.enhancedText}
			disabled={adv.badges.enhancedHidden}
			placeholder="Geliştirilmiş"
			clearButton={false}
		/>
	</div>
	<div class="row2">
		<div class="field">
			<TextBlock variant="caption">Başlangıç</TextBlock>
			<TextBox bind:value={adv.badges.enhancedFrom} disabled={!adv.badges.on} clearButton={false} />
		</div>
		<div class="field">
			<TextBlock variant="caption">Bitiş</TextBlock>
			<TextBox bind:value={adv.badges.enhancedTo} disabled={!adv.badges.on} clearButton={false} />
		</div>
	</div>
</Section>
{/if}

{#if show("scrollbar")}
<Section icon="scrollbar" title="Kaydırma çubuğu" onReset={reset("scrollbar")}>
	<StatusBar
		severity="information"
		title="Sitenin kendi API'si kullanılıyor"
		message="Site OverlayScrollbars kütüphanesini kullanır ve varsayılan ::-webkit-scrollbar stil tanımlarını devre dışı bırakır. Buradaki ayarlar sitenin doğrudan okuduğu --os-* değişkenlerini günceller."
		closable={false}
	/>
	<ToggleSwitch bind:checked={adv.scrollbar.on}>Kaydırma çubuğunu özelleştir</ToggleSwitch>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Kalınlık — {adv.scrollbar.size}px</TextBlock>
		<Slider bind:value={adv.scrollbar.size} min={2} max={24} step={1} disabled={!adv.scrollbar.on} suffix="px" />
	</label>
	<ColorField
		spec={{
			token: "--os-handle-bg",
			label: "Tutamak rengi",
			hint: "Sitede --fds-control-strong-fill-default'tan geliyor",
			alpha: true,
			defaultAlpha: 54
		}}
		bind:hex={adv.scrollbar.handle.hex}
		bind:alpha={adv.scrollbar.handle.alpha}
		disabled={!adv.scrollbar.on}
	/>
	<ColorField
		spec={{
			token: "--os-track-bg-hover",
			label: "Ray rengi (hover)",
			hint: "Sitede --fds-layer-background-default'tan geliyor",
			alpha: true,
			defaultAlpha: 30
		}}
		bind:hex={adv.scrollbar.track.hex}
		bind:alpha={adv.scrollbar.track.alpha}
		disabled={!adv.scrollbar.on}
	/>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Tutamak yuvarlaklığı — {adv.scrollbar.handleRadius}px</TextBlock>
		<Slider bind:value={adv.scrollbar.handleRadius} min={0} max={50} step={1} disabled={!adv.scrollbar.on} suffix="px" />
	</label>
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Ray yuvarlaklığı — {adv.scrollbar.trackRadius}px</TextBlock>
		<Slider bind:value={adv.scrollbar.trackRadius} min={0} max={50} step={1} disabled={!adv.scrollbar.on} suffix="px" />
	</label>
</Section>
{/if}

<slot name="interaction" />

{#if show("links")}
<Section icon="link" title="Bağlantılar" onReset={reset("links")}>
	<StatusBar
		severity="information"
		title="Vurgudan bağımsız"
		message="Site bağlantıları --fds-accent-* rampasından değil, ayrı bir --fds-accent-text-* setinden boyuyor (.button.style-hyperlink). Yani vurgu rengini değiştirmeden bağlantıları ayrı renklendirebilirsiniz."
		closable={false}
	/>
	<ToggleSwitch bind:checked={adv.links.on}>Bağlantı renklerini özelleştir</ToggleSwitch>
	{#each LINK_TOKENS as spec, i}
		<ColorField
			{spec}
			bind:hex={adv.links.colors[i].hex}
			bind:alpha={adv.links.colors[i].alpha}
			disabled={!adv.links.on}
		/>
	{/each}
</Section>
{/if}

{#if show("system")}
<Section icon="system" title="Uyarı ve bildirim" onReset={reset("system")}>
	<TextBlock variant="caption">
		Bildirim rozetleri ve sistem uyarı renkleri.
	</TextBlock>
	<ToggleSwitch bind:checked={adv.system.on}>Durum renklerini özelleştir</ToggleSwitch>
	{#each SYSTEM_TOKENS as spec, i}
		<ColorField
			{spec}
			bind:hex={adv.system.colors[i].hex}
			bind:alpha={adv.system.colors[i].alpha}
			disabled={!adv.system.on}
		/>
	{/each}
</Section>
{/if}



{#if grouped}<h3 class="group">Gelişmiş</h3>{/if}

<slot name="raw" />

<style>
	/*
	   Grup başlığı: bölümleri gruplayan tek görsel işaret. Kutu ya da çizgi
	   değil, yalnızca küçük ve sönük bir etiket — yirmi bölümlük listeye
	   yirmi kutu daha eklemek sadeleştirmenin tersi olurdu.
	*/
	.group {
		margin: 8px 0 0;
		font-size: var(--fds-caption-font-size);
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--fds-text-secondary);
	}

	.group:first-child {
		margin-top: 0;
	}

	/* Açılır listenin görünümü `SelectField.svelte`'te. */

	.row {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.row2 {
		display: flex;
		gap: 8px;
	}

	.row2 > .field {
		flex: 1 1 0;
		min-width: 0;
	}

	.row-between {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.gap {
		margin-left: 6px;
	}

	.preview {
		width: 100%;
		max-height: 90px;
		object-fit: cover;
		border-radius: var(--fds-control-corner-radius);
		border: 1px solid var(--fds-control-stroke-default);
	}

	.thumb {
		height: 28px;
		width: 28px;
		object-fit: contain;
	}
</style>
