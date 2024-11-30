extends Control
var mt_node = MeshtasticNode.new()
var radio_settings:String = ""
var connected = false

@onready var connect_radio = $organizer_box/connection_box/connect_radio
@onready var conn_options = $organizer_box/connection_box/connection_options
@onready var selected_type = conn_options.get_item_text(0)
@onready var serial_menu = $organizer_box/connection_box/Serial_Options
@onready var ip_input = $organizer_box/connection_box/Ip_input
@onready var text_client = $organizer_box/TabContainer/"Text Client"
@onready var tic_tac_toe = $"organizer_box/TabContainer/Sub-port Tic-Tac-Toe"


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for port in MeshtasticNode.list_ports():
		serial_menu.add_item(port['name'])
	get_tree().get_root().size_changed.connect(resize)


func get_short_name(id):
	var node_info = mt_node.get_node_info(id)
	var short_name = node_info.get('short_name', 'Unknown')
	return short_name


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	# Poll node and check its state and update ui elements accordingly
	var res = mt_node.poll()
	connected = res
	if res and !connect_radio.disabled:  # we are connected
		connect_radio.disabled = true
	elif not res and connect_radio.disabled:
		connect_radio.disabled = false
		
	# Replace with checkups for message states
	if mt_node.get_available_messages():
		var msg = mt_node.get_message()
		# print(msg)
		# print(mt_node.get_node_info(msg['source']))
		var port_num = msg.get('port_num', 'Unknown')
		match port_num:
			"TEXT_MESSAGE_APP":
				var encoded_msg: PackedByteArray = msg['payload']
				var decoded_msg = encoded_msg.get_string_from_utf8()
				add_chat_message(msg['source'], decoded_msg)
			"PRIVATE_APP":
				var encoded_msg: PackedByteArray = msg['payload']
				var short_name = get_short_name(msg['source'])
				tic_tac_toe.process_message(short_name, msg['sub_port'], encoded_msg)


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

func add_chat_message(node_id: String, msg):
	if node_id:
		var node_info = mt_node.get_node_info(node_id)
		var short_name = node_info.get('short_name', 'Unknown')
		text_client.add_chat_message(short_name, msg)

func resize():
	self.size = get_tree().get_root().size

func _on_text_client_new_text_to_send(new_text: String) -> void:
	if mt_node.is_open() and new_text:
		mt_node.send_text_message(new_text, 0, 0, true)
		add_chat_message(mt_node.get_connected_id(), new_text)


func _on_subport_tic_tac_toe_new_data_to_send(sub_port: int, data: PackedByteArray) -> void:
	if mt_node.is_open() and len(data):
		mt_node.send_raw_message(data, 0, 0, sub_port, false)
