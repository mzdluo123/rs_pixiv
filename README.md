# rs_piviv

易于部署的pixiv代理服务

# 快速开始

https://hub.docker.com/r/rainchan2/rs-pixiv/tags

```shell
docker run -it --restart=always -d --name rs-px -p 127.0.0.1:4568:8080 \
	-e PIXIV_UID=你的UID \
	-e PIXIV_COOKIE=Base64编码的cookie字符串 \
	rainchan2/rs-pixiv:版本号
```