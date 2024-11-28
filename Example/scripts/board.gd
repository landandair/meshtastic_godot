extends GridContainer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass

func process_move(move, is_x):
	print(move)  # Process move here and check win
	
	return check_win()

func check_win():
	return true
