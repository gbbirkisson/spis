HTMX_VERSION:=2.0.2
HYPERSCRIPT_VERSION:=0.9.12

all: htmx-${HTMX_VERSION}.js hyperscript-${HYPERSCRIPT_VERSION}.js

htmx-${HTMX_VERSION}.js:
	wget https://unpkg.com/htmx.org@${HTMX_VERSION}/dist/htmx.min.js -O htmx-${HTMX_VERSION}.js

hyperscript-${HYPERSCRIPT_VERSION}.js:
	wget https://unpkg.com/hyperscript.org@${HYPERSCRIPT_VERSION} -O hyperscript-${HYPERSCRIPT_VERSION}.js
