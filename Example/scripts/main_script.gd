extends Node2D
var mt_node = MeshtasticNode.new()


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	print(mt_node.list_ports())
	mt_node.open_serial_node("/dev/tty.usbserial-54760041581")
	mt_node.send_text_message("Hi from godot", 0, 0, false)
	#mt_node.send_raw_message("hi".to_ascii_buffer(), 0, 0, 10, false)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	mt_node.poll()
	if mt_node.get_available_messages():
		var msg = mt_node.get_message()
		print(msg)
		print(msg['source'])
		print(mt_node.get_node_info(msg['source']))
		
