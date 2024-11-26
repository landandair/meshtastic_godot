extends Control
var mt_node = MeshtasticNode.new()
var radio_settings:String = ""

@onready var conn_options = $connection_options
@onready var selected_type = conn_options.get_item_text(0)
@onready var serial_menu = $Serial_Options
@onready var ip_input = $Ip_input


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for port in MeshtasticNode.list_ports():
		serial_menu.add_item(port['name'])
	#mt_node.open_serial_node("/dev/tty.usbserial-54760041581")
	#mt_node.send_text_message("Hi from godot", 0, 0, false)
	#mt_node.send_raw_message("hi".to_ascii_buffer(), 0, 0, 10, false)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
	mt_node.poll()
	if mt_node.get_available_messages():
		var msg = mt_node.get_message()
		print(msg)
		print(msg['source'])
		print(mt_node.get_node_info(msg['source']))
		


func _on_connect_radio_pressed() -> void:
	if conn_options.get_item_text(0) == selected_type:
		mt_node.open_serial_node(radio_settings)
		mt_node.send_text_message("Hi from godot", 0, 0, false)
	elif conn_options.get_item_text(1) == selected_type:
		var slots = radio_settings.split(":")
		if len(slots) == 2:
			var ip = slots[0]
			var port = int(slots[1])
			mt_node.open_tcp_node(ip, port)
			mt_node.send_text_message("Hi from godot", 0, 0, false)
			


func _on_connection_options_item_selected(index: int) -> void:
	var item = conn_options.get_item_text(index)
	selected_type = item
	ip_input.visible = !ip_input.visible
	serial_menu.visible = !serial_menu.visible

	


func _on_serial_option_item_selected(index: int) -> void:
	radio_settings = serial_menu.get_item_text(index)


func _on_ip_input_text_submitted(new_text: String) -> void:
	radio_settings = new_text
