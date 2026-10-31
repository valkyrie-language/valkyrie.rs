<widget settings>
<Column class="settings">
    <Text class="settings-title">Preferences</Text>
    <Text>Theme: {theme}</Text>
    <Text>Auto start: {auto_start}</Text>
    <Button @click="reset">Reset</Button>
    <Link href="index.html">Back</Link>
</Column>
</widget>
<script>
let theme: string = "light"
let mut auto_start: bool = true
micro reset() {
    auto_start = false
}
</script>
<style>
.settings { padding: 24px; }
.settings-title { font-size: 18px; }
</style>
