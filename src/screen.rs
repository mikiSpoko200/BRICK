/// Normalized screen.


/*
 3D to 2D transformation:
 Field of view:
^ z
|  -1 ___________ +1  -- end plane
|     \  |_|    /
|      \   |   /
|       \  |  /
|     -1 \_|_/ +1  -- start plane
           ^-- angle represents FOV.

Note: Narrowing the field of view would give an impression of zooming.
      This may seem counter intuitive at first but it's because normally we don't stretch
      what we zoom at. Not humans anyway, think of it in geometric sense.
      After zooming the objects far away will take more of screen and screen preserves it's size.
      LIKE IN MINECRAFT!!
      This *scaling factor* can be related to the theta - angle of the VOF.
      It is assumed that we know how far an object is.
      In that case we can use trigonometry of the right triangle to determine the said scaling factor.

Note: What do want to achieve?
      Well we would like to INCREASE the amount of thing we CAN SEE. Again think about the geometric
      camera frustum. There is subtle issue here. If we naively increase theta and scale by tangent
      we just SPREAD | ELONGATE | EXTEND the end place without actually bringing any new objects into
      FOV. That is why we inverse the tangent. We effectively bring MORE STUFF into our normalized space.

Note: What coordinates does this transform affect?
      This requires us to define a coordinate system and pick a reference point.
      Since in 2D graphics we use x and y which are width and height of our screen
      it is natural to leave them as is and just add the third dimension as z axis pointing
      in direction perpendicular to the width and height; which is depth.
      Having established that we can answer our original question.
      Only width and height are affected by the VOF. We move more things from left and right inside,
      as well as, from the top and bottom. Depth however stays the same.

Note: how to normalize *z*?
        ^ z
        |  -1 ___________ +1  -- end plane
        |     \  |_|    /
        |      \   |   /
        |       \  |  /
        |     -1 \_|_/ +1  -- start plane - this plane does not start at 0 since players head in not directly in the screen.
        |          .     <-- head is a bit further from the monitor, and we can adjust VOF depending on that distance for the most natural experience.
        z

*/
