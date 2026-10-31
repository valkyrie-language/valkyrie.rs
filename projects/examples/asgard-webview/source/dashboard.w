<widget dashboard>
<Column class="dashboard">
    <Text class="dashboard-title">{heading}</Text>
    <Box class="status-box">
        <Text>{status}</Text>
    </Box>
</Column>
</widget>
<script>
let heading: string = "Workspace status"
let status: string = "All services online"
</script>
<style>
.dashboard { padding: 24px; }
.status-box { border: 1px solid #cccccc; padding: 12px; }
</style>
