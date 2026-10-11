Type "cat demo.toml"
Enter
Sleep 2s
Type "pcmp demo.toml --out charts"
Enter
Wait@20s /^\$\s*$/
Sleep 1500ms
Type "pcmp demo.toml --kind bars --out charts"
Enter
Wait@20s /^\$\s*$/
Sleep 1500ms
Type "file charts/demo-paired.png"
Enter
Wait@10s /^\$\s*$/
Sleep 3s
