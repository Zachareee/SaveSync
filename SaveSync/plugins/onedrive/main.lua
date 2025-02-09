function Info()
	return {
		name = "OneDrive",
		description = "OneDrive plugin",
		author = "Zachareee",
	}
end

function Init(credentials)
	print(credentials)

	-- simulate delay of authenticating
	return "ONEDRIVECREDENTIALS"
end

function Abort()
	print("Onedrive has been aborted")
	return "Unsuccessful abort"
end

--------------------------
--- Sync function
--- @param zipbuffer table
function Sync(zipbuffer)
	print("Received Sync call")
	print(zipbuffer)

	local buffer = string.char(table.unpack(zipbuffer))
	print(buffer)

	local file = io.open("fake.zip", "wb")
	if file then
		file:write(buffer)
		file:close()
	end
end
