SDK_VER=25.09.29
URL=https://www.qhyccd.com/file/repository/publish/SDK/${SDK_VER}/sdk_linux64_${SDK_VER}.tgz

.PHONY: download-linux-sdk

download-linux-sdk:
	rm -rf /tmp/qhy-linux64
	curl -o /tmp/qhy-linux64.tgz ${URL}
	mkdir -p /tmp/qhy-linux64
	tar -xf /tmp/qhy-linux64.tgz -C /tmp/qhy-linux64
	cp /tmp/qhy-linux64/sdk_linux64_${SDK_VER}/usr/local/lib/* vendored/camera/linux/x64
	cp /tmp/qhy-linux64/sdk_linux64_${SDK_VER}/usr/local/include/* libqhy-sys/include/

.PHONY: publish-libqhy

publish-libqhy:
	cargo publish --dry-run -p libqhy && cargo publish -p libqhy
