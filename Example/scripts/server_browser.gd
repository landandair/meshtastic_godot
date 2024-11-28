extends VBoxContainer

signal joinGame(name:String, port:int)

@export var serverInfo : PackedScene
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	new_server_message("test", 1234)  # REMOVE


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass

func new_server_message(name: String, port):
	for child in get_children():
		if name == child.name:
			return
	var currentInfo = serverInfo.instantiate()
	currentInfo.name = name
	currentInfo.get_node("name").text = name
	currentInfo.get_node("port").text = str(port)
	currentInfo.joinGame.connect(join_server)
	add_child(currentInfo)

func join_server(name, port):
	joinGame.emit(name, port)
