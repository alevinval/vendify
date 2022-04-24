# Example usage

To generate the following example, the following commands have been executed:

```
vendor init

[INFO] initializing vendor in current directory
[INFO] .vendor.yml has been created
```

```
vendor add https://github.com/alevinval/ledger --targets pkg/proto --targets README.md --extensions proto

[INFO] added dependency https://github.com/alevinval/ledger@master
```

```
vendor install

[INFO] installing https://github.com/alevinval/ledger@master
[INFO] 	.../README.md -> vendor/README.md
[INFO] 	.../pkg/proto/ledger.proto -> vendor/pkg/proto/ledger.proto
[INFO] 	.../pkg/proto/checkpoint.proto -> vendor/pkg/proto/checkpoint.proto
[INFO] 	🔒 04abf50e06ae0dcf88e75cb35e922c7ae3aefad6
[INFO] install success ✅
```
