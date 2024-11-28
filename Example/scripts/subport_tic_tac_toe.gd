extends VBoxContainer

signal new_data_to_send(sub_port: int, data: PackedByteArray)
var message_queue = []
var connected_server = -1
var move_number = 0
var is_turn = false
var game_started = false
var need_ack = false
var retries = 3
var attempts = 0
var last_message = PackedByteArray()
var last_port = 0
var host_str = "TicS".to_ascii_buffer()
var join_str = "TicJ".to_ascii_buffer()
var ackjoin_str = "TicA".to_ascii_buffer()
var move_str = "TicM".to_ascii_buffer()
var ack_move = "Tica".to_ascii_buffer()

@onready var sub_port = $server_settings/sub_port
@onready var host = $server_settings/host
@onready var advertisement = $server_advertisment
@onready var server_browser = $game_server_browser/server_browser
@onready var board = $game_server_browser/CenterContainer/board

func _ready() -> void:
	sub_port.value = randi_range(0, 65535)


func process_message(name, port: int, data: PackedByteArray):
	var prefix = data.slice(0, 4)
	var remains = data.slice(4)
	match prefix:
		host_str:  # Discovered a server
			server_browser.new_server_message(name, port)
		join_str:  # Received request to join hosted server
			if port == sub_port.value and not advertisement.is_stopped():
				print("Someone joined server")
				is_turn = false
				host.button_pressed = false
				advertisement.stop()
				send_data(port, ackjoin_str, true)
		ackjoin_str:  # Received ack from our join message
			if port == connected_server:
				print("Ack of join received: We joined server")
				advertisement.stop()  # Stop hosting if started
				host.button_pressed = false
				need_ack = false
				is_turn = true
				# Prepare to send first move
		move_str:  # Received move from player
			if port == connected_server and not is_turn:
				print("We received a move")
				if len(remains) == 2:
					var move_num = remains[0]
					var move = remains[-1]
					if move_num == move_number:
						board.process_move(int(move))
						send_data(port, ack_move+remains)
						is_turn = true
						# Prepare to send first move
		ack_move:
			if port == connected_server:
				print("Ack move stop retransmitting last move")
				need_ack = false
			


func send_data(port: int, data: PackedByteArray, needack: bool=false):
	if needack:
		last_message = data
		last_port = port
		need_ack = needack
		attempts = 0
	new_data_to_send.emit(port, data)


func _on_server_browser_join_game(name: String, port: int) -> void:
	send_data(port, join_str)
	connected_server = port


func _on_retransmission_timer_timeout() -> void:
	if need_ack and attempts < retries:
		attempts += 1
		send_data(last_port, last_message)


func _on_server_advertisment_timeout() -> void:
	advertise_server()


func advertise_server():
	var port = sub_port.value
	send_data(port, host_str)


func _on_host_toggled(toggled_on: bool) -> void:
	if toggled_on:
		advertisement.start()
		advertise_server()
	else:
		advertisement.stop()
