FROM cosmos/gaia:latest

COPY --chmod=755 entry-point.sh /data/entry-point.sh

EXPOSE 9090
EXPOSE 1317
EXPOSE 26656
EXPOSE 26657

ENTRYPOINT ["/data/entry-point.sh"]