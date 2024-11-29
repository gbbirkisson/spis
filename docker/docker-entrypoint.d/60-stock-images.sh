#!/usr/bin/env sh

if [ ! -z "$SPIS_MEDIA_FETCH_STOCK" ]; then
    SPIS_MEDIA_STOCK_DIR="$SPIS_MEDIA_DIR/stock"

    if [ ! -d "$SPIS_MEDIA_STOCK_DIR" ]; then
        echo "Starting to fetch $SPIS_MEDIA_FETCH_STOCK stock images"

        mkdir -p "$SPIS_MEDIA_STOCK_DIR"

        for i in $(seq $SPIS_MEDIA_FETCH_STOCK); do
            curl -s -L -o "$SPIS_MEDIA_STOCK_DIR/${i}.jpg" https://picsum.photos/800/600
        done

        echo "Done fetching $SPIS_MEDIA_FETCH_STOCK stock images"
    fi
fi
