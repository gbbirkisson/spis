HTMX_VERSION:=2.0.7
HYPERSCRIPT_VERSION:=0.9.14

all: htmx-${HTMX_VERSION}.js hyperscript-${HYPERSCRIPT_VERSION}.js

htmx-${HTMX_VERSION}.js:
	wget https://unpkg.com/htmx.org@${HTMX_VERSION}/dist/htmx.min.js -O htmx-${HTMX_VERSION}.js

hyperscript-${HYPERSCRIPT_VERSION}.js:
	wget https://unpkg.com/hyperscript.org@${HYPERSCRIPT_VERSION}/dist/_hyperscript.min.js -O hyperscript-${HYPERSCRIPT_VERSION}.js
