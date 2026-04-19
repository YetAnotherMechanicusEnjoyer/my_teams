SERVER_BIN	=	myteams_server

CLIENT_BIN	=	myteams_cli

all:
	@cargo build --release
	@cp target/release/$(SERVER_BIN) .
	@cp target/release/$(CLIENT_BIN) .

clean:
	@cargo clean

fclean:	clean
	@rm -f $(SERVER_BIN)
	@rm -f $(CLIENT_BIN)

re:	fclean	all

.PHONY: all clean fclean re
