<widget index>
<Column class="desk">
    <Text class="desk-title">{title}</Text>
    <Text class="desk-subtitle">{subtitle}</Text>
    <Text class="desk-status">{status}</Text>
    <Text class="desk-count">Count: {count}</Text>
    <Button @click="on_increment">Increment</Button>
</Column>
</widget>
<script>
let title: utf8 = "Valkyrie Desk"
let subtitle: utf8 = "WebView desktop host example"
let status: utf8 = "All services online"
let mut count: i32 = 0
micro on_increment() {
    count = count + 1
}
</script>
<style>
.desk { padding: 24px; }
.desk-title { font-size: 20px; }
.desk-count { font-family: monospace; }
</style>
