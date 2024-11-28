extends Control
var mt_node = MeshtasticNode.new()
var radio_settings:String = ""
var connected = false

@onready var connect_radio = $organizer_box/connection_box/connect_radio
@onready var conn_options = $organizer_box/connection_box/connection_options
@onready var selected_type = conn_options.get_item_text(0)
@onready var serial_menu = $organizer_box/connection_box/Serial_Options
@onready var ip_input = $organizer_box/connection_box/Ip_input
@onready var chat_box = $organizer_box/chat_box
@onready var chat_msg = $organizer_box/chat_message


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for port in MeshtasticNode.list_ports():
		serial_menu.add_item(port['name'])
	get_tree().get_root().size_changed.connect(resize)
	#mt_node.send_raw_message("hi".to_ascii_buffer(), 0, 0, 10, false)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	# Poll node and check its state and update ui elements accordingly
	var res = mt_node.poll()
	connected = res
	if res and !connect_radio.disabled:  # we are connected
		connect_radio.disabled = true
	elif connect_radio.disabled:
		connect_radio.disabled = false
		
	# Replace with checkups for message states
	if mt_node.get_available_messages():
		var msg = mt_node.get_message()
		print(msg)
		print(msg['source'])
		print(mt_node.get_node_info(msg['source']))
		var port_num = msg.get('port_num', 'Unknown')
		match port_num:
			"TEXT_MESSAGE_APP":
				var encoded_msg: PackedByteArray = msg['payload']
				var decoded_msg = encoded_msg.get_string_from_utf8()
				add_chat_message(msg['source'], decoded_msg)


func _on_connect_radio_pressed() -> void:
	if conn_options.get_item_text(0) == selected_type:
		mt_node.open_serial_node(radio_settings)
	elif conn_options.get_item_text(1) == selected_type:
		var slots = radio_settings.split(":")
		if len(slots) == 2:
			var ip = slots[0]
			var port = int(slots[1])
			mt_node.open_tcp_node(ip, port)
			


func _on_connection_options_item_selected(index: int) -> void:
	var item = conn_options.get_item_text(index)
	selected_type = item
	ip_input.visible = !ip_input.visible
	serial_menu.visible = !serial_menu.visible


func _on_serial_option_item_selected(index: int) -> void:
	radio_settings = serial_menu.get_item_text(index)


func _on_ip_input_text_submitted(new_text: String) -> void:
	radio_settings = new_text

"""Add chat message to main channel"""
func _on_chat_message_text_submitted(new_text: String) -> void:
	if mt_node.is_open() and new_text:
		mt_node.send_text_message(new_text, 0, 0, true)
		add_chat_message(mt_node.get_connected_id(), new_text)
	chat_msg.clear()
		

func add_chat_message(node_id: String, msg):
	if node_id:
		var node_info = mt_node.get_node_info(node_id)
		var short_name = node_info.get('short_name', 'Unknown')
		chat_box.text += str(short_name, ": ", msg, "\n")
		chat_box.scroll_vertical = chat_box.get_line_height()
	
func resize():
	self.size = get_tree().get_root().size
